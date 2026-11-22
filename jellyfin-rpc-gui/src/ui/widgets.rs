use dioxus::prelude::*;

#[component]
pub fn Section(title: String, children: Element) -> Element {
    rsx! {
        div { class: "section",
            h3 { "{title}" }
            {children}
        }
    }
}

#[component]
pub fn Help(text: String) -> Element {
    rsx! { p { class: "help", "{text}" } }
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
    let mut reveal = use_signal(|| false);
    let kind = if number {
        "number"
    } else if secret && !reveal() {
        "password"
    } else {
        "text"
    };

    rsx! {
        label { class: "field",
            span { class: "label", "{label}" }
            div { class: "input-row",
                input {
                    r#type: "{kind}",
                    value: "{value}",
                    placeholder: "{placeholder}",
                    spellcheck: false,
                    oninput: move |e| oninput.call(e.value()),
                    onfocus: move |_| onfocus.call(()),
                }
                if secret {
                    button {
                        class: "btn small",
                        r#type: "button",
                        onclick: move |_| reveal.toggle(),
                        if reveal() { "Hide" } else { "Show" }
                    }
                }
            }
            if !help.is_empty() {
                span { class: "help", "{help}" }
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
) -> Element {
    rsx! {
        label { class: "toggle",
            input {
                r#type: "checkbox",
                checked: checked,
                onchange: move |e| onchange.call(e.checked()),
            }
            span { class: "toggle-text",
                span { class: "label", "{label}" }
                if !help.is_empty() {
                    span { class: "help", "{help}" }
                }
            }
        }
    }
}

/// Row of mutually exclusive buttons. Options are `(key, label)`.
#[component]
pub fn Choice(
    options: Vec<(String, String)>,
    value: String,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "choice",
            for (key, label) in options {
                button {
                    key: "{key}",
                    r#type: "button",
                    class: if key == value { "choice-opt active" } else { "choice-opt" },
                    onclick: {
                        let key = key.clone();
                        move |_| onchange.call(key.clone())
                    },
                    "{label}"
                }
            }
        }
    }
}

/// Editable list of strings shown as chips, with clickable suggestions.
#[component]
pub fn StringList(
    label: String,
    items: Vec<String>,
    onchange: EventHandler<Vec<String>>,
    #[props(default)] placeholder: String,
    #[props(default)] suggestions: Vec<String>,
    #[props(default)] help: String,
) -> Element {
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
            span { class: "label", "{label}" }
            div { class: "chips",
                for (i, item) in items.iter().enumerate() {
                    span { key: "{item}", class: "chip",
                        "{item}"
                        button {
                            r#type: "button",
                            class: "chip-x",
                            onclick: {
                                let items = items.clone();
                                move |_| {
                                    let mut next = items.clone();
                                    next.remove(i);
                                    onchange.call(next);
                                }
                            },
                            "x"
                        }
                    }
                }
            }
            div { class: "input-row",
                input {
                    r#type: "text",
                    value: "{draft}",
                    placeholder: "{placeholder}",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            add_enter(draft());
                        }
                    },
                }
                button {
                    class: "btn small",
                    r#type: "button",
                    onclick: move |_| add_click(draft()),
                    "Add"
                }
            }
            if !remaining.is_empty() {
                div { class: "chips suggestions",
                    for s in remaining {
                        button {
                            key: "{s}",
                            r#type: "button",
                            class: "chip suggestion",
                            onclick: {
                                let s = s.clone();
                                let mut add = add_suggestion.clone();
                                move |_| add(s.clone())
                            },
                            "+ {s}"
                        }
                    }
                }
            }
            if !help.is_empty() {
                span { class: "help", "{help}" }
            }
        }
    }
}
