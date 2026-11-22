use jellyfin_rpc::{
    config::ConfigBuilder, is_jellyfin_error, Client, ClientBuilder, JfError, NowPlayingItem,
    PresencePreview,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, RecvTimeoutError, Sender},
        Arc,
    },
    time::{Duration, Instant},
};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::sync::watch;

const INTERVAL: Duration = Duration::from_secs(7);

/// Process names (lowercase, without `.exe`) of the official discord clients.
const DISCORD_PROCESSES: &[&str] = &["discord", "discordptb", "discordcanary", "discorddevelopment"];

/// A library item sent to discord instead of the live session, to test a configuration.
#[derive(Clone, Debug)]
pub struct TestPresence {
    pub item: NowPlayingItem,
    pub paused: bool,
    pub position_secs: u64,
}

pub enum Command {
    Reload(Box<ConfigBuilder>),
    SetPaused(bool),
    SetTest(Option<Box<TestPresence>>),
    Quit,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AgentStatus {
    pub discord_running: bool,
    pub ipc_connected: bool,
    pub paused: bool,
    /// A test item is displayed instead of the live session.
    pub test_active: bool,
    pub config_error: Option<String>,
    pub error: Option<String>,
    /// Discord processes cannot be inspected, so discord is treated as not running.
    pub detection_error: Option<String>,
    pub now_playing: String,
    /// What is displayed on discord, or would be if discord was running.
    pub preview: Option<PresencePreview>,
}

#[derive(Clone)]
pub struct AgentHandle {
    commands: Sender<Command>,
    pub status: watch::Receiver<AgentStatus>,
    stopped: Arc<AtomicBool>,
}

impl AgentHandle {
    pub fn send(&self, command: Command) {
        let _ = self.commands.send(command);
    }

    /// Clears the activity and stops the agent, waiting a short while for it to finish.
    pub fn shutdown(&self) {
        self.send(Command::Quit);
        let start = Instant::now();
        while !self.stopped.load(Ordering::SeqCst) && start.elapsed() < Duration::from_secs(2) {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

pub fn spawn(config: ConfigBuilder, urls_path: String) -> AgentHandle {
    let (commands, rx) = channel();
    let (tx, status) = watch::channel(AgentStatus::default());
    let stopped = Arc::new(AtomicBool::new(false));

    let worker = Worker::new(config, urls_path, tx);
    let stopped_flag = stopped.clone();

    std::thread::spawn(move || {
        worker.run(rx);
        stopped_flag.store(true, Ordering::SeqCst);
    });

    AgentHandle {
        commands,
        status,
        stopped,
    }
}

/// Whether an official discord client is running. Errors when processes cannot be inspected,
/// in which case discord is never assumed to be running.
fn discord_running(sys: &mut Option<System>) -> Result<bool, String> {
    if sys.is_none() {
        *sys = Some(System::new().map_err(|e| format!("Cannot detect Discord processes: {e}"))?);
    }

    let sys = sys.as_mut().expect("initialized above");

    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());

    Ok(sys.processes().values().any(|process| {
        let name = process.name().to_string_lossy().to_lowercase();
        DISCORD_PROCESSES.contains(&name.strip_suffix(".exe").unwrap_or(&name))
    }))
}

fn build_client(config: &ConfigBuilder, urls_path: &str) -> Result<Client, String> {
    if config.jellyfin.url.trim().is_empty() || config.jellyfin.api_key.trim().is_empty() {
        return Err("Set the Jellyfin URL and API key to get started".to_string());
    }

    ClientBuilder::from_config(config.clone().build(), urls_path)
        .build()
        .map_err(|e| e.to_string())
}

struct Worker {
    config: ConfigBuilder,
    urls_path: String,
    client: Option<Client>,
    sys: Option<System>,
    connected: bool,
    shown: bool,
    test: Option<(TestPresence, Instant)>,
    status: AgentStatus,
    tx: watch::Sender<AgentStatus>,
}

impl Worker {
    fn new(config: ConfigBuilder, urls_path: String, tx: watch::Sender<AgentStatus>) -> Self {
        let mut worker = Self {
            config,
            urls_path,
            client: None,
            sys: None,
            connected: false,
            shown: false,
            test: None,
            status: AgentStatus::default(),
            tx,
        };
        worker.rebuild_client();
        worker
    }

    fn rebuild_client(&mut self) {
        self.connected = false;
        self.shown = false;
        self.status.ipc_connected = false;

        match build_client(&self.config, &self.urls_path) {
            Ok(client) => {
                self.client = Some(client);
                self.status.config_error = None;
            }
            Err(err) => {
                self.client = None;
                self.status.config_error = Some(err);
                self.status.preview = None;
                self.status.now_playing.clear();
            }
        }
    }

    /// The test item with its position advanced by the time it has been displayed.
    fn test_presence(&self) -> Option<TestPresence> {
        let (test, since) = self.test.as_ref()?;
        let mut test = test.clone();

        if !test.paused {
            let duration = test.item.run_time_ticks.unwrap_or(0).max(0) as u64 / 10_000_000;
            test.position_secs += since.elapsed().as_secs();
            if duration > 0 {
                test.position_secs = test.position_secs.min(duration);
            }
        }

        Some(test)
    }

    fn clear_activity(&mut self) {
        if let (Some(client), true) = (self.client.as_mut(), self.shown) {
            let _ = client.clear_activity();
        }
        self.shown = false;
    }

    fn run(mut self, rx: Receiver<Command>) {
        loop {
            self.tick();
            self.publish();

            let deadline = Instant::now() + INTERVAL;

            // Wait for the next tick, handling commands as they arrive
            loop {
                match rx.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                    Ok(Command::Quit) | Err(RecvTimeoutError::Disconnected) => {
                        self.clear_activity();
                        return;
                    }
                    Ok(Command::SetPaused(paused)) => {
                        self.status.paused = paused;
                        if paused {
                            self.clear_activity();
                        }
                        break;
                    }
                    Ok(Command::SetTest(test)) => {
                        self.status.test_active = test.is_some();
                        self.test = test.map(|t| (*t, Instant::now()));
                        if self.test.is_none() {
                            self.clear_activity();
                        }
                        break;
                    }
                    Ok(Command::Reload(config)) => {
                        self.clear_activity();
                        self.config = *config;
                        self.rebuild_client();
                        break;
                    }
                    Err(RecvTimeoutError::Timeout) => break,
                }
            }
        }
    }

    fn publish(&self) {
        self.tx.send_if_modified(|current| {
            let changed = *current != self.status;
            if changed {
                *current = self.status.clone();
            }
            changed
        });
    }

    fn tick(&mut self) {
        match discord_running(&mut self.sys) {
            Ok(running) => {
                self.status.discord_running = running;
                self.status.detection_error = None;
            }
            Err(err) => {
                if self.status.detection_error.is_none() {
                    log::warn!("{err}");
                }
                self.status.discord_running = false;
                self.status.detection_error = Some(err);
            }
        }

        let test = self.test_presence();

        let Some(client) = self.client.as_mut() else {
            return;
        };

        if self.status.paused {
            self.status.preview = None;
            self.status.now_playing.clear();
            self.status.error = None;
            return;
        }

        // Only talk to discord while it is running
        if !self.status.discord_running {
            if self.connected {
                log::info!("Discord closed, dropping IPC connection");
                self.rebuild_client();
                return self.tick_preview_only();
            }
            return self.tick_preview_only();
        }

        if !self.connected {
            match client.connect() {
                Ok(()) => {
                    log::info!("Connected to Discord");
                    self.connected = true;
                    self.status.ipc_connected = true;
                }
                Err(err) => {
                    self.status.error = Some(format!("Could not connect to Discord: {err}"));
                    return self.tick_preview_only();
                }
            }
        }

        let result = match test {
            Some(t) => client.set_activity_item(t.item, t.paused, t.position_secs),
            None => client.set_activity(),
        };

        match result {
            Ok(activity) => {
                if activity.is_empty() && self.shown {
                    let _ = client.clear_activity();
                    self.shown = false;
                } else if !activity.is_empty() {
                    self.shown = true;
                }
                self.status.now_playing = activity;
                self.status.preview = client.current_preview().cloned();
                self.status.error = None;
            }
            Err(err) => {
                self.status.preview = None;
                self.status.now_playing.clear();

                match err.downcast_ref::<JfError>() {
                    Some(JfError::ContentBlacklist | JfError::UnrecognizedMediaType) => {
                        self.status.error = None;
                        if self.shown {
                            let _ = client.clear_activity();
                            self.shown = false;
                        }
                    }
                    _ if is_jellyfin_error(&*err) => {
                        self.status.error = Some(format!("Jellyfin: {err}"));
                    }
                    _ => {
                        log::error!("{err}");
                        self.status.error = Some(format!("Discord: {err}"));
                        self.rebuild_client();
                    }
                }
            }
        }
    }

    /// Without discord there is no activity to set, but the preview still follows the
    /// live jellyfin session.
    fn tick_preview_only(&mut self) {
        let test = self.test_presence();

        let Some(client) = self.client.as_mut() else {
            return;
        };

        self.status.now_playing.clear();

        let result = match test {
            Some(t) => client
                .preview_item(t.item, t.paused, t.position_secs)
                .map(|p| (!p.is_blacklisted && !p.hidden_when_paused).then_some(p)),
            None => client.preview_live(),
        };

        match result {
            Ok(preview) => {
                self.status.preview = preview;
                if self.status.error.as_deref().is_some_and(|e| e.starts_with("Jellyfin")) {
                    self.status.error = None;
                }
            }
            Err(err) => {
                self.status.preview = None;
                if is_jellyfin_error(&*err) {
                    self.status.error = Some(format!("Jellyfin: {err}"));
                }
            }
        }
    }
}
