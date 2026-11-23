use std::{
    collections::HashSet,
    sync::atomic::{AtomicUsize, Ordering},
};

use dioxus::prelude::*;
use dioxus_icons::lucide::X;

use crate::components::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card, CardContent, CardHeader,
    CardTitle, Input, Item, ItemActions, ItemContent, ItemSize, ItemVariant, Label, Switch,
    ToggleGroup, ToggleItem,
};

/// A unique element id, used to tie a label to its control.
fn use_field_id() -> String {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    use_hook(|| format!("field-{}", NEXT.fetch_add(1, Ordering::Relaxed)))
}

#[component]
pub fn Section(title: String, children: Element) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "{title}" }
            }
            CardContent {
                div { class: "section-body", {children} }
            }
        }
    }
}

#[component]
pub fn Help(text: String) -> Element {
    rsx! { p { class: "help", "{text}" } }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BannerKind {
    Error,
    Ok,
    Test,
}

/// Full width notice. `action` is shown at the end of the row.
#[component]
pub fn Banner(kind: BannerKind, children: Element, action: Option<Element>) -> Element {
    let class = match kind {
        BannerKind::Error => "banner error",
        BannerKind::Ok => "banner ok",
        BannerKind::Test => "banner test",
    };

    rsx! {
        Item { variant: ItemVariant::Outline, size: ItemSize::Sm, class,
            ItemContent { {children} }
            if let Some(action) = action {
                ItemActions { {action} }
            }
        }
    }
}

#[component]
pub fn TextField(
    label: String,
    value: String,
    oninput: EventHandler<String>,
    #[props(default)] placeholder: String,
    #[props(default)] help: String,
    #[props(default)] secret: bool,
    #[props(default)] number: bool,
    #[props(default)] onfocus: EventHandler<()>,
) -> Element {
    let id = use_field_id();
    let mut reveal = use_signal(|| false);
    let kind = if number {
        "number"
    } else if secret && !reveal() {
        "password"
    } else {
        "text"
    };

    rsx! {
        div { class: "field",
            Label { html_for: id.clone(), "{label}" }
            div { class: "input-row",
                Input {
                    id: "{id}",
                    r#type: "{kind}",
                    value: "{value}",
                    placeholder: "{placeholder}",
                    oninput: move |e: FormEvent| oninput.call(e.value()),
                    onfocus: move |_: FocusEvent| onfocus.call(()),
                }
                if secret {
                    Button {
                        variant: ButtonVariant::Outline,
                        r#type: "button",
                        onclick: move |_| reveal.toggle(),
                        if reveal() { "Hide" } else { "Show" }
                    }
                }
            }
            if !help.is_empty() {
                Help { text: help }
            }
        }
    }
}

#[component]
pub fn Toggle(
    label: String,
    checked: bool,
    onchange: EventHandler<bool>,
    #[props(default)] help: String,
    #[props(default)] disabled: bool,
) -> Element {
    let id = use_field_id();

    rsx! {
        div { class: "toggle",
            Switch {
                id: "{id}",
                checked: Some(checked),
                disabled,
                on_checked_change: move |v| onchange.call(v),
            }
            div { class: "toggle-text",
                Label { html_for: id.clone(), "{label}" }
                if !help.is_empty() {
                    Help { text: help }
                }
            }
        }
    }
}

/// Row of mutually exclusive options. Options are `(key, label)`.
#[component]
pub fn Choice(
    options: Vec<(String, String)>,
    value: String,
    onchange: EventHandler<String>,
    #[props(default)] label: String,
) -> Element {
    let id = use_field_id();
    let opts = options.clone();
    let pressed: HashSet<usize> = options
        .iter()
        .position(|(key, _)| *key == value)
        .into_iter()
        .collect();

    rsx! {
        div { class: "field",
            if !label.is_empty() {
                Label { html_for: id.clone(), "{label}" }
            }
            ToggleGroup {
                id: "{id}",
                horizontal: true,
                allow_multiple_pressed: false,
                pressed: Some(pressed),
                on_pressed_change: move |set: HashSet<usize>| {
                    if let Some(&i) = set.iter().next() {
                        if let Some((key, _)) = opts.get(i) {
                            onchange.call(key.clone());
                        }
                    }
                },
                for (i, (key, label)) in options.iter().enumerate() {
                    ToggleItem {
                        key: "{key}",
                        index: i,
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Small button that inserts or adds the thing it names.
#[component]
pub fn Suggestion(text: String, onclick: EventHandler<()>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            size: ButtonSize::Xs,
            r#type: "button",
            onclick: move |_| onclick.call(()),
            "{text}"
        }
    }
}

/// Editable list of strings shown as badges, with clickable suggestions.
#[component]
pub fn StringList(
    label: String,
    items: Vec<String>,
    onchange: EventHandler<Vec<String>>,
    #[props(default)] placeholder: String,
    #[props(default)] suggestions: Vec<String>,
    #[props(default)] help: String,
) -> Element {
    let id = use_field_id();
    let mut draft = use_signal(String::new);

    let add = {
        let items = items.clone();
        move |value: String| {
            let value = value.trim().to_string();
            if !value.is_empty() && !items.contains(&value) {
                let mut next = items.clone();
                next.push(value);
                onchange.call(next);
            }
            draft.set(String::new());
        }
    };
    let mut add_enter = add.clone();
    let mut add_click = add.clone();
    let add_suggestion = add.clone();

    let remaining: Vec<String> = suggestions
        .into_iter()
        .filter(|s| !items.contains(s))
        .collect();

    rsx! {
        div { class: "field",
            Label { html_for: id.clone(), "{label}" }
            if !items.is_empty() {
                div { class: "chips",
                    for (i, item) in items.iter().enumerate() {
                        Badge { key: "{item}", variant: BadgeVariant::Secondary,
                            "{item}"
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::IconXs,
                                r#type: "button",
                                style: "width: 1rem; height: 1rem;",
                                aria_label: "Remove {item}",
                                onclick: {
                                    let items = items.clone();
                                    move |_| {
                                        let mut next = items.clone();
                                        next.remove(i);
                                        onchange.call(next);
                                    }
                                },
                                X {}
                            }
                        }
                    }
                }
            }
            div { class: "input-row",
                Input {
                    id: "{id}",
                    r#type: "text",
                    value: "{draft}",
                    placeholder: "{placeholder}",
                    oninput: move |e: FormEvent| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            add_enter(draft());
                        }
                    },
                }
                Button {
                    variant: ButtonVariant::Outline,
                    r#type: "button",
                    onclick: move |_| add_click(draft()),
                    "Add"
                }
            }
            if !remaining.is_empty() {
                div { class: "chips",
                    for s in remaining {
                        Suggestion {
                            key: "{s}",
                            text: "+ {s}",
                            onclick: {
                                let s = s.clone();
                                let mut add = add_suggestion.clone();
                                move |_| add(s.clone())
                            },
                        }
                    }
                }
            }
            if !help.is_empty() {
                Help { text: help }
            }
        }
    }
}
