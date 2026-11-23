use dioxus::prelude::*;
use dioxus_icons::lucide::ChevronRight;
use jellyfin_rpc::{
    fetch_application_name, LibraryFolder, MediaType, NowPlayingItem, PresencePreview,
    PreviewActivityType, StatusType, DEFAULT_APPLICATION_ID,
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{
    agent::{Command, TestPresence},
    app::AppState,
    components::{
        Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Input, Item, ItemActions,
        ItemContent, ItemDescription, ItemGroup, ItemMedia, ItemMediaVariant, ItemSize, ItemTitle,
        ItemVariant, Label, ScrollArea, Skeleton, Slider,
    },
    form::Form,
    ui::widgets::{Choice, Toggle},
    util::{blocking, client_for, thumbnail_url},
    Startup,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Live,
    Library,
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn format_time(secs: i64) -> String {
    let secs = secs.max(0);
    let (h, m, s) = (secs / 3600, secs / 60 % 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

fn media_types(filter: &str) -> Vec<MediaType> {
    match filter {
        "movie" => vec![MediaType::Movie],
        "episode" => vec![MediaType::Episode],
        "music" => vec![MediaType::Music],
        "audiobook" => vec![MediaType::AudioBook],
        "book" => vec![MediaType::Book],
        _ => Vec::new(),
    }
}

/// One step down the series, season, episode hierarchy.
#[derive(Clone, PartialEq)]
struct Crumb {
    id: String,
    name: String,
    is_series: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Target {
    Root,
    /// Go back to an earlier step of the trail.
    Crumb(usize),
    /// Open a folder from the current list.
    Open,
}

/// Folders and playable items shown in the browser.
type Listing = (Vec<LibraryFolder>, Vec<NowPlayingItem>);

fn load_children(form: Form, crumb: Crumb) -> Result<Listing, String> {
    let client = client_for(&form)?;
    let err = |e: Box<dyn std::error::Error>| e.to_string();

    if crumb.is_series {
        let seasons = client.list_seasons(&crumb.id).map_err(err)?;
        // Nothing to pick from with a single season
        if seasons.len() > 1 {
            return Ok((seasons, Vec::new()));
        }
    }

    Ok((Vec::new(), client.list_episodes(&crumb.id).map_err(err)?))
}

fn folder_subtitle(folder: &LibraryFolder, is_series: bool) -> String {
    if is_series {
        folder
            .production_year
            .map(|y| y.to_string())
            .unwrap_or_default()
    } else {
        folder
            .index_number
            .map(|n| format!("Season {n}"))
            .unwrap_or_default()
    }
}

fn subtitle(item: &NowPlayingItem) -> String {
    match item.media_type {
        MediaType::Episode => format!(
            "{} S{}E{}",
            item.series_name.clone().unwrap_or_default(),
            item.parent_index_number.unwrap_or(0),
            item.index_number.unwrap_or(0)
        ),
        MediaType::Music => format!(
            "{} - {}",
            item.artists
                .as_ref()
                .and_then(|a| a.first().cloned())
                .unwrap_or_default(),
            item.album.clone().unwrap_or_default()
        ),
        _ => item
            .production_year
            .map(|y| y.to_string())
            .unwrap_or_default(),
    }
}

fn image_id(item: &NowPlayingItem) -> String {
    match item.media_type {
        MediaType::Music => item.album_id.clone().unwrap_or_else(|| item.id.clone()),
        _ => item.id.clone(),
    }
}

fn duration_secs(item: &NowPlayingItem) -> u64 {
    item.run_time_ticks.unwrap_or(0).max(0) as u64 / 10_000_000
}

#[component]
pub fn PreviewPanel() -> Element {
    let state = use_context::<AppState>();
    let agent = use_context::<Startup>().agent;
    let form = state.form;

    let mut source = use_signal(|| Source::Live);
    let mut query = use_signal(String::new);
    let mut filter = use_signal(String::new);
    let mut root = use_signal(Listing::default);
    let mut listing = use_signal(Listing::default);
    let mut nav = use_signal(Vec::<Crumb>::new);
    let mut searching = use_signal(|| false);
    let mut search_error = use_signal(|| None::<String>);
    let mut selected = use_signal(|| None::<NowPlayingItem>);
    let mut paused = use_signal(|| false);
    let mut position = use_signal(|| 0u64);

    let search = move || {
        spawn(async move {
            searching.set(true);
            search_error.set(None);

            let f = form();
            let q = query();
            let filter = filter();

            let out = blocking(move || -> Result<Listing, String> {
                let client = client_for(&f)?;
                let err = |e: Box<dyn std::error::Error>| e.to_string();

                // Episodes are reached through their series
                let folders = if filter.is_empty() || filter == "episode" {
                    client.search_series(&q, 30).map_err(err)?
                } else {
                    Vec::new()
                };

                let items = if filter == "episode" {
                    Vec::new()
                } else {
                    let types = if filter.is_empty() {
                        vec![
                            MediaType::Movie,
                            MediaType::Music,
                            MediaType::AudioBook,
                            MediaType::Book,
                        ]
                    } else {
                        media_types(&filter)
                    };
                    client
                        .search_library_items(&q, &types, 30)
                        .map_err(|e| e.to_string())?
                };

                Ok((folders, items))
            })
            .await;

            match out {
                Ok(found) => {
                    root.set(found.clone());
                    listing.set(found);
                    nav.write().clear();
                }
                Err(err) => search_error.set(Some(err)),
            }

            searching.set(false);
        });
    };

    let browse = move |target: Target, folder: Option<LibraryFolder>| {
        spawn(async move {
            let crumb = match (target, folder) {
                (Target::Root, _) => {
                    listing.set(root());
                    nav.write().clear();
                    return;
                }
                (Target::Open, None) => return,
                (Target::Crumb(_), Some(_)) => return,
                (Target::Crumb(i), None) => {
                    let Some(crumb) = nav.read().get(i).cloned() else {
                        return;
                    };
                    crumb
                }
                (Target::Open, Some(folder)) => Crumb {
                    id: folder.id,
                    name: folder.name,
                    is_series: nav.read().is_empty(),
                },
            };

            searching.set(true);
            search_error.set(None);

            let f = form();
            let c = crumb.clone();
            let out = blocking(move || load_children(f, c)).await;

            match out {
                Ok(children) => {
                    listing.set(children);
                    match target {
                        Target::Crumb(i) => nav.write().truncate(i + 1),
                        _ => nav.write().push(crumb),
                    }
                }
                Err(err) => search_error.set(Some(err)),
            }

            searching.set(false);
        });
    };

    let preview = use_resource(move || async move {
        let item = selected()?;
        let f = form();
        let is_paused = paused();
        let pos = position();

        // Debounce, the resource is restarted on every change
        tokio::time::sleep(Duration::from_millis(300)).await;

        Some(
            blocking(move || {
                client_for(&f)?
                    .preview_item(item, is_paused, pos)
                    .map_err(|e| e.to_string())
            })
            .await,
        )
    });

    // Keep the test presence in sync with the selection while it is being sent
    let sync_agent = agent.clone();
    use_resource(move || {
        let agent = sync_agent.clone();
        async move {
            let item = selected()?;
            let (is_paused, position_secs) = (paused(), position());

            tokio::time::sleep(Duration::from_millis(400)).await;

            if state.status.peek().test_active {
                agent.send(Command::SetTest(Some(Box::new(TestPresence {
                    item,
                    paused: is_paused,
                    position_secs,
                }))));
            }
            Some(())
        }
    });

    // The name of the discord application is what discord shows as the activity name
    let app_name = use_resource(move || async move {
        let id = form.read().application_id.trim().to_string();
        let id = if id.is_empty() {
            DEFAULT_APPLICATION_ID.to_string()
        } else {
            id
        };

        tokio::time::sleep(Duration::from_millis(500)).await;
        blocking(move || fetch_application_name(&id).ok()).await
    });
    let app_name = (*app_name.read_unchecked())
        .clone()
        .flatten()
        .unwrap_or_else(|| "Jellyfin".to_string());

    let status = (state.status)();
    let dirty = *state.form.read() != *state.saved.read();
    let can_test = status.discord_running && !status.paused && status.config_error.is_none();
    let base_url = form.read().url.clone();
    let source_key = match source() {
        Source::Live => "live",
        Source::Library => "library",
    }
    .to_string();

    rsx! {
        div { class: "preview-head",
            h2 { "Discord preview" }
            Choice {
                options: vec![
                    ("live".to_string(), "Live session".to_string()),
                    ("library".to_string(), "Library item".to_string()),
                ],
                value: source_key,
                onchange: move |key: String| {
                    source.set(if key == "library" { Source::Library } else { Source::Live })
                },
            }
        }
        match source() {
            Source::Live => rsx! {
                match status.preview.clone() {
                    Some(p) => rsx! {
                        div {
                            Badge {
                                variant: BadgeVariant::Primary,
                                style: if status.test_active { "background-color: var(--warn);" } else { "background-color: var(--ok); color: white;" },
                                if status.test_active { "TEST" } else { "LIVE" }
                            }
                        }
                        PresenceCard { preview: p, app_name: app_name.clone() }
                    },
                    None => rsx! {
                        div { class: "empty",
                            if let Some(err) = &status.config_error {
                                p { "{err}" }
                            } else if status.paused {
                                p { "RPC is paused." }
                            } else {
                                p { "Nothing would be shown on Discord right now." }
                                p { class: "help", "Start playing something on Jellyfin as a configured user and it appears here, mirroring the real presence. Use Library item to preview any title." }
                            }
                        }
                    },
                }
            },
            Source::Library => rsx! {
                div { class: "search",
                    div { class: "input-row",
                        Input {
                            r#type: "text",
                            value: "{query}",
                            placeholder: "Search your library",
                            oninput: move |e: FormEvent| query.set(e.value()),
                            onkeydown: move |e: KeyboardEvent| {
                                if e.key() == Key::Enter {
                                    search();
                                }
                            },
                        }
                        Button {
                            variant: ButtonVariant::Secondary,
                            disabled: searching(),
                            onclick: move |_| search(),
                            if searching() { "Searching..." } else { "Search" }
                        }
                    }
                    Choice {
                        options: vec![
                            (String::new(), "All".to_string()),
                            ("movie".to_string(), "Movies".to_string()),
                            ("episode".to_string(), "Episodes".to_string()),
                            ("music".to_string(), "Music".to_string()),
                            ("audiobook".to_string(), "Audiobooks".to_string()),
                            ("book".to_string(), "Books".to_string()),
                        ],
                        value: filter(),
                        onchange: move |key: String| filter.set(key),
                    }
                    if let Some(err) = search_error() {
                        p { class: "result error", "{err}" }
                    }
                    if !nav.read().is_empty() {
                        div { class: "crumbs",
                            Button {
                                variant: ButtonVariant::Link,
                                size: ButtonSize::Xs,
                                r#type: "button",
                                onclick: move |_| browse(Target::Root, None),
                                "Results"
                            }
                            for (i, crumb) in nav().into_iter().enumerate() {
                                span { key: "{crumb.id}", class: "crumb-sep", "/" }
                                Button {
                                    variant: ButtonVariant::Link,
                                    size: ButtonSize::Xs,
                                    r#type: "button",
                                    onclick: move |_| browse(Target::Crumb(i), None),
                                    "{crumb.name}"
                                }
                            }
                        }
                    }
                    ScrollArea { style: "max-height: 260px;",
                        ItemGroup {
                            for folder in listing().0 {
                                ResultRow {
                                    key: "{folder.id}",
                                    title: folder.name.clone(),
                                    subtitle: folder_subtitle(&folder, nav.read().is_empty()),
                                    image: thumbnail_url(&base_url, &folder.id),
                                    folder: true,
                                    onselect: {
                                        let folder = folder.clone();
                                        move |_| browse(Target::Open, Some(folder.clone()))
                                    },
                                }
                            }
                            for item in listing().1 {
                                ResultRow {
                                    key: "{item.id}",
                                    title: item.name.clone(),
                                    subtitle: format!("{} {}", item.media_type, subtitle(&item)),
                                    image: thumbnail_url(&base_url, &image_id(&item)),
                                    active: selected().as_ref().is_some_and(|s| s.id == item.id),
                                    onselect: {
                                        let item = item.clone();
                                        move |_| {
                                            position.set(duration_secs(&item) / 4);
                                            selected.set(Some(item.clone()));
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
                if let Some(item) = selected() {
                    div { class: "sim",
                        Toggle {
                            label: "Paused",
                            checked: paused(),
                            onchange: move |v| paused.set(v),
                        }
                        if duration_secs(&item) > 0 {
                            div { class: "field",
                                Label { html_for: "preview-position", "Position {format_time(position() as i64)} / {format_time(duration_secs(&item) as i64)}" }
                                Slider {
                                    id: "preview-position",
                                    label: Some("Position".to_string()),
                                    value: Some(position() as f64),
                                    min: 0.0,
                                    max: duration_secs(&item) as f64,
                                    step: 1.0,
                                    on_value_change: move |v: f64| position.set(v as u64),
                                }
                            }
                        }
                    }
                    match &*preview.read_unchecked() {
                        Some(Some(Ok(p))) => rsx! { PresenceCard { preview: p.clone(), app_name: app_name.clone() } },
                        Some(Some(Err(err))) => rsx! { p { class: "result error", "{err}" } },
                        _ => rsx! { PresenceSkeleton {} },
                    }
                    div { class: "test-rpc",
                        Toggle {
                            label: "Send this to Discord as the real RPC",
                            help: "Replaces your live session on Discord until switched off, for testing.",
                            checked: status.test_active,
                            disabled: !can_test && !status.test_active,
                            onchange: {
                                let agent = agent.clone();
                                let item = item.clone();
                                move |on: bool| {
                                    agent.send(Command::SetTest(on.then(|| {
                                        Box::new(TestPresence {
                                            item: item.clone(),
                                            paused: paused(),
                                            position_secs: position(),
                                        })
                                    })));
                                }
                            },
                        }
                        if !can_test && !status.test_active {
                            p { class: "help", "Needs Discord running, a valid configuration and RPC not paused." }
                        }
                        if dirty {
                            p { class: "help", "The real RPC uses your saved settings. Save & Apply to test your latest changes." }
                        }
                    }
                } else {
                    div { class: "empty",
                        p { "Search your Jellyfin library and select an item to see how it would look with the current, unsaved settings." }
                    }
                }
            },
        }
    }
}

/// A folder or playable item in the library browser.
#[component]
fn ResultRow(
    title: String,
    subtitle: String,
    image: String,
    onselect: EventHandler<()>,
    #[props(default)] active: bool,
    #[props(default)] folder: bool,
) -> Element {
    rsx! {
        Item {
            variant: if active { ItemVariant::Muted } else { ItemVariant::Default },
            size: ItemSize::Sm,
            class: "result-row",
            role: "button",
            tabindex: "0",
            onclick: move |_| onselect.call(()),
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Enter {
                    onselect.call(());
                }
            },
            ItemMedia { variant: ItemMediaVariant::Image,
                img { src: "{image}" }
            }
            ItemContent {
                ItemTitle { "{title}" }
                ItemDescription { "{subtitle}" }
            }
            if folder {
                ItemActions {
                    ChevronRight { size: "16px" }
                }
            }
        }
    }
}

/// Placeholder with the shape of [`PresenceCard`], shown while a preview is loading.
#[component]
fn PresenceSkeleton() -> Element {
    rsx! {
        div { class: "member-list", aria_busy: "true", aria_label: "Loading preview",
            Skeleton { style: "width: 70px; height: 12px;" }
            Skeleton { style: "width: 160px; height: 12px;" }
        }
        div { class: "card", aria_hidden: "true",
            Skeleton { style: "width: 120px; height: 12px;" }
            div { class: "card-body",
                Skeleton { style: "width: 88px; height: 88px; border-radius: 8px; flex: none;" }
                div { class: "card-text", style: "gap: 8px;",
                    Skeleton { style: "width: 70%; height: 14px;" }
                    Skeleton { style: "width: 50%; height: 14px;" }
                    Skeleton { style: "width: 100%; height: 4px; margin-top: 6px;" }
                }
            }
        }
    }
}

#[component]
pub fn PresenceCard(preview: PresencePreview, app_name: String) -> Element {
    let mut now = use_signal(unix_now);

    use_future(move || async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            now.set(unix_now());
        }
    });

    let verb = match preview.activity_type {
        PreviewActivityType::Playing => "Playing",
        PreviewActivityType::Watching => "Watching",
        PreviewActivityType::Listening => "Listening to",
    };
    let headline = match preview.status_display_type {
        StatusType::Name => app_name.clone(),
        StatusType::State => preview.state.clone(),
        StatusType::Details => preview.details.clone(),
    };
    let image = preview
        .large_image_data_uri
        .clone()
        .unwrap_or_else(|| preview.large_image_url.clone());
    let image_title = preview.large_image_text.clone().unwrap_or_default();

    let progress = preview.timestamps.map(|(start, end)| {
        let total = (end - start).max(1);
        (now().saturating_sub(start).clamp(0, total), total)
    });

    rsx! {
        if preview.is_blacklisted {
            div { class: "banner error", "Blocked by your blacklist, Discord would show nothing." }
        }
        if preview.hidden_when_paused {
            div { class: "banner error", "Paused with \"Show activity while paused\" off, Discord would show nothing." }
        }
        div { class: "member-list",
            span { class: "member-label", "Member list" }
            span { class: "member-status", "{verb} {headline}" }
        }
        div { class: "card",
            div { class: "card-head", "{verb} {app_name}" }
            div { class: "card-body",
                div { class: "art",
                    img { class: "large", src: "{image}" }
                    if !image_title.is_empty() {
                        div { class: "tip", "{image_title}" }
                    }
                    if let Some(small) = &preview.small_image_url {
                        img { class: "small", src: "{small}" }
                        if let Some(text) = &preview.small_image_text {
                            div { class: "tip small-tip", "{text}" }
                        }
                    }
                }
                div { class: "card-text",
                    div { class: "card-title", "{preview.details}" }
                    div { class: "card-line", "{preview.state}" }
                    if let Some((elapsed, total)) = progress {
                        div { class: "progress",
                            span { class: "time", "{format_time(elapsed)}" }
                            div { class: "bar",
                                div { class: "fill", style: "width: {elapsed * 100 / total}%" }
                            }
                            span { class: "time", "{format_time(total)}" }
                        }
                    }
                }
            }
            for (i, button) in preview.buttons.iter().enumerate() {
                if button.url.starts_with("http://") || button.url.starts_with("https://") {
                    a {
                        key: "{i}",
                        class: "card-button",
                        href: "{button.url}",
                        target: "_blank",
                        "{button.name}"
                    }
                } else {
                    div { key: "{i}", class: "card-button disabled", "{button.name}" }
                }
            }
        }
    }
}
