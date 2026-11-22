use dioxus::prelude::*;

use crate::{
    agent::{AgentHandle, AgentStatus, Command},
    form::Form,
    tray::{status_text, use_tray},
    ui::{
        discord::DiscordTab, display::DisplayTab, images::ImagesTab, jellyfin::JellyfinTab,
        preview::PreviewPanel,
    },
    Startup,
};

const CSS: &str = include_str!("../assets/main.css");

/// Shared state, available to every component through context.
#[derive(Clone, Copy)]
pub struct AppState {
    pub form: Signal<Form>,
    pub saved: Signal<Form>,
    pub status: Signal<AgentStatus>,
    pub message: Signal<Option<(bool, String)>>,
    /// Users and libraries fetched from the jellyfin server, offered as suggestions.
    pub users: Signal<Vec<String>>,
    pub libraries: Signal<Vec<String>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Jellyfin,
    Display,
    Discord,
    Images,
}

impl Tab {
    const ALL: [Tab; 4] = [Tab::Jellyfin, Tab::Display, Tab::Discord, Tab::Images];

    fn label(self) -> &'static str {
        match self {
            Tab::Jellyfin => "Jellyfin",
            Tab::Display => "Display",
            Tab::Discord => "Discord",
            Tab::Images => "Images",
        }
    }
}

fn save(state: AppState, startup: &Startup) {
    let AppState {
        form,
        mut saved,
        mut message,
        ..
    } = state;

    let current = form.read().clone();
    let builder = current.to_builder();

    match builder.save(&startup.config_path) {
        Ok(()) => {
            saved.set(current);
            startup.agent.send(Command::Reload(Box::new(builder)));
            message.set(Some((true, format!("Saved to {}", startup.config_path))));
        }
        Err(err) => message.set(Some((false, format!("Could not save: {err}")))),
    }
}

#[component]
pub fn App() -> Element {
    let startup = use_context::<Startup>();
    let agent: AgentHandle = startup.agent.clone();

    let initial = Form::from_builder(&startup.builder);
    let form = use_signal(|| initial.clone());
    let saved = use_signal(|| initial.clone());
    let mut status = use_signal(|| agent.status.borrow().clone());
    let message = use_signal(|| None);
    let users = use_signal(Vec::new);
    let libraries = use_signal(Vec::new);

    let state = use_context_provider(|| AppState {
        form,
        saved,
        status,
        message,
        users,
        libraries,
    });

    let rx = agent.status.clone();
    use_future(move || {
        let mut rx = rx.clone();
        async move {
            while rx.changed().await.is_ok() {
                status.set(rx.borrow_and_update().clone());
            }
        }
    });

    use_tray(agent.clone(), status);

    let mut tab = use_signal(|| Tab::Jellyfin);
    let dirty = *form.read() != *saved.read();
    let current = status();
    let paused = current.paused;

    let (dot, text) = if current.config_error.is_some() {
        ("dot warn", status_text(&current))
    } else if current.discord_running && current.ipc_connected {
        ("dot ok", status_text(&current))
    } else {
        ("dot idle", status_text(&current))
    };

    let discord_label = if current.discord_running {
        "Discord running"
    } else {
        "Discord not running"
    };

    let save_startup = startup.clone();
    let pause_agent = agent.clone();
    let stop_agent = agent.clone();

    rsx! {
        style { dangerous_inner_html: CSS }
        div { class: "app",
            header { class: "topbar",
                div { class: "brand", "Jellyfin-RPC" }
                div { class: "status",
                    span { class: "{dot}" }
                    span { class: "status-text", "{text}" }
                    span { class: if current.discord_running { "pill on" } else { "pill" }, "{discord_label}" }
                }
                div { class: "actions",
                    button {
                        class: "btn",
                        onclick: move |_| pause_agent.send(Command::SetPaused(!paused)),
                        if paused { "Resume RPC" } else { "Pause RPC" }
                    }
                    button {
                        class: "btn primary",
                        disabled: !dirty,
                        onclick: move |_| save(state, &save_startup),
                        if dirty { "Save & Apply" } else { "Saved" }
                    }
                }
            }
            if let Some(err) = &startup.load_error {
                div { class: "banner error",
                    "Could not read the existing config at {startup.config_path}: {err}. Saving will overwrite it."
                }
            }
            if let Some(err) = &current.config_error {
                if !err.starts_with("Set the Jellyfin") {
                    div { class: "banner error", "Configuration error: {err}" }
                }
            }
            if current.test_active {
                div { class: "banner test",
                    span { "A test presence is being sent to Discord instead of your live session." }
                    button {
                        class: "btn small",
                        onclick: move |_| stop_agent.send(Command::SetTest(None)),
                        "Stop"
                    }
                }
            }
            if let Some(err) = &current.detection_error {
                div { class: "banner error", "{err}" }
            }
            if let Some(err) = &current.error {
                div { class: "banner error", "{err}" }
            }
            if let Some((ok, msg)) = message() {
                div { class: if ok { "banner ok" } else { "banner error" }, "{msg}" }
            }
            main { class: "content",
                section { class: "config",
                    nav { class: "tabs",
                        for t in Tab::ALL {
                            button {
                                key: "{t.label()}",
                                class: if tab() == t { "tab active" } else { "tab" },
                                onclick: move |_| tab.set(t),
                                "{t.label()}"
                            }
                        }
                    }
                    div { class: "tab-body",
                        match tab() {
                            Tab::Jellyfin => rsx! { JellyfinTab {} },
                            Tab::Display => rsx! { DisplayTab {} },
                            Tab::Discord => rsx! { DiscordTab {} },
                            Tab::Images => rsx! { ImagesTab {} },
                        }
                    }
                }
                aside { class: "preview", PreviewPanel {} }
            }
        }
    }
}
