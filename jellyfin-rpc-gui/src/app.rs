use dioxus::prelude::*;

use crate::{
    agent::{AgentHandle, AgentStatus, Command},
    components::{
        Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Separator, TabContent, TabList,
        TabTrigger, Tabs,
    },
    form::Form,
    tray::{status_text, use_tray},
    ui::{
        discord::DiscordTab,
        display::DisplayTab,
        images::ImagesTab,
        jellyfin::JellyfinTab,
        preview::PreviewPanel,
        widgets::{Banner, BannerKind},
    },
    Startup,
};

use crate::embedded::{
    components::CSS as COMPONENTS_CSS, FONT_SORA_LATIN_EXT_WGHT_NORMAL,
    FONT_SORA_LATIN_WGHT_NORMAL, FONT_SPLINE_SANS_MONO_LATIN_EXT_WGHT_NORMAL,
    FONT_SPLINE_SANS_MONO_LATIN_WGHT_NORMAL,
};

const CSS: &str = include_str!("../assets/main.css");
const COMPONENTS_THEME: &str = include_str!("../assets/dx-components-theme.css");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASES_URL: &str = "https://github.com/LeviSnoot/jellyfin-rpc/releases/latest";

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

    fn id(self) -> &'static str {
        match self {
            Tab::Jellyfin => "jellyfin",
            Tab::Display => "display",
            Tab::Discord => "discord",
            Tab::Images => "images",
        }
    }

    fn index(self) -> usize {
        match self {
            Tab::Jellyfin => 0,
            Tab::Display => 1,
            Tab::Discord => 2,
            Tab::Images => 3,
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

    let mut tab = use_signal(|| Some("jellyfin".to_string()));
    let dirty = *form.read() != *saved.read();
    let current = status();
    let paused = current.paused;

    let (dot, text) = if current.config_error.is_some() || current.paused {
        ("dot warn", status_text(&current))
    } else if current.discord_running && current.ipc_connected {
        ("dot ok", status_text(&current))
    } else {
        ("dot idle", status_text(&current))
    };

    let save_startup = startup.clone();
    let pause_agent = agent.clone();
    let stop_agent = agent.clone();

    let fonts = format!(
        "@font-face {{ font-family: 'Sora Variable'; font-style: normal; font-display: swap; font-weight: 100 800; \
         src: url('{FONT_SORA_LATIN_EXT_WGHT_NORMAL}') format('woff2-variations'); \
         unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF; }} \
         @font-face {{ font-family: 'Sora Variable'; font-style: normal; font-display: swap; font-weight: 100 800; \
         src: url('{FONT_SORA_LATIN_WGHT_NORMAL}') format('woff2-variations'); \
         unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD; }} \
         @font-face {{ font-family: 'Spline Sans Mono Variable'; font-style: normal; font-display: swap; font-weight: 300 700; \
         src: url('{FONT_SPLINE_SANS_MONO_LATIN_EXT_WGHT_NORMAL}') format('woff2-variations'); \
         unicode-range: U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,U+2C60-2C7F,U+A720-A7FF; }} \
         @font-face {{ font-family: 'Spline Sans Mono Variable'; font-style: normal; font-display: swap; font-weight: 300 700; \
         src: url('{FONT_SPLINE_SANS_MONO_LATIN_WGHT_NORMAL}') format('woff2-variations'); \
         unicode-range: U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD; }}"
    );

    rsx! {
        style { dangerous_inner_html: fonts }
        style { dangerous_inner_html: COMPONENTS_THEME }
        style { dangerous_inner_html: COMPONENTS_CSS }
        style { dangerous_inner_html: CSS }
        div { class: "app",
            header { class: "topbar",
                div { class: "brand", "Jellyfin-RPC" }
                div { class: "status",
                    Badge {
                        variant: BadgeVariant::Secondary,
                        style: "max-width: 100%; overflow: hidden;",
                        span { class: "{dot}" }
                        span { class: "status-text", "{text}" }
                    }
                }
                div { class: "actions",
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| pause_agent.send(Command::SetPaused(!paused)),
                        if paused { "Resume RPC" } else { "Pause RPC" }
                    }
                    Button {
                        variant: ButtonVariant::Primary,
                        disabled: !dirty,
                        onclick: move |_| save(state, &save_startup),
                        if dirty { "Save & Apply" } else { "Saved" }
                    }
                }
            }
            if let Some(err) = &startup.load_error {
                Banner { kind: BannerKind::Error,
                    "Could not read the existing config at {startup.config_path}: {err}. Saving will overwrite it."
                }
            }
            if let Some(err) = &current.config_error {
                if !err.starts_with("Set the Jellyfin") {
                    Banner { kind: BannerKind::Error, "Configuration error: {err}" }
                }
            }
            if current.test_active {
                Banner {
                    kind: BannerKind::Test,
                    action: rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            size: ButtonSize::Sm,
                            onclick: move |_| stop_agent.send(Command::SetTest(None)),
                            "Stop"
                        }
                    },
                    "A test presence is being sent to Discord instead of your live session."
                }
            }
            if let Some(err) = &current.detection_error {
                Banner { kind: BannerKind::Error, "{err}" }
            }
            if let Some(err) = &current.error {
                Banner { kind: BannerKind::Error, "{err}" }
            }
            if let Some((ok, msg)) = message() {
                Banner { kind: if ok { BannerKind::Ok } else { BannerKind::Error }, "{msg}" }
            }
            main { class: "content",
                section { class: "config",
                    Tabs {
                        class: "config-tabs",
                        value: tab,
                        horizontal: true,
                        on_value_change: move |v: String| tab.set(Some(v)),
                        TabList {
                            for t in Tab::ALL {
                                TabTrigger {
                                    key: "{t.id()}",
                                    value: t.id().to_string(),
                                    index: t.index(),
                                    "{t.label()}"
                                }
                            }
                        }
                        for t in Tab::ALL {
                            TabContent {
                                key: "{t.id()}-content",
                                class: Some("tab-body".to_string()),
                                value: t.id().to_string(),
                                index: t.index(),
                                match t {
                                    Tab::Jellyfin => rsx! { JellyfinTab {} },
                                    Tab::Display => rsx! { DisplayTab {} },
                                    Tab::Discord => rsx! { DiscordTab {} },
                                    Tab::Images => rsx! { ImagesTab {} },
                                }
                            }
                        }
                    }
                }
                aside { class: "preview", PreviewPanel {} }
            }
            footer { class: "version-footer",
                Badge { variant: BadgeVariant::Outline,
                    "v{VERSION}"
                    Separator { horizontal: false, decorative: true, style: "height: 10px;" }
                    a { href: RELEASES_URL, "Check for new releases" }
                }
            }
        }
    }
}
