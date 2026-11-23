use dioxus::prelude::*;

use crate::{
    app::AppState,
    form::{DisplayForm, DisplayMode, Form},
    ui::widgets::{Choice, Help, Section, Suggestion, TextField, Toggle},
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Music,
    Movies,
    Episodes,
}

impl Kind {
    fn title(self) -> &'static str {
        match self {
            Kind::Music => "Music",
            Kind::Movies => "Movies",
            Kind::Episodes => "Episodes",
        }
    }

    fn placeholders(self) -> &'static [&'static str] {
        match self {
            Kind::Music => &[
                "track", "album", "artists", "genres", "year", "version", "sep",
            ],
            Kind::Movies => &[
                "title",
                "original-title",
                "genres",
                "year",
                "critic-score",
                "community-score",
                "version",
                "sep",
            ],
            Kind::Episodes => &[
                "show-title",
                "title",
                "original-title",
                "episode",
                "episode-padded",
                "season",
                "season-padded",
                "year",
                "genres",
                "studio",
                "version",
                "sep",
            ],
        }
    }

    fn get(self, form: &Form) -> &DisplayForm {
        match self {
            Kind::Music => &form.music,
            Kind::Movies => &form.movies,
            Kind::Episodes => &form.episodes,
        }
    }

    fn get_mut(self, form: &mut Form) -> &mut DisplayForm {
        match self {
            Kind::Music => &mut form.music,
            Kind::Movies => &mut form.movies,
            Kind::Episodes => &mut form.episodes,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Line {
    Details,
    State,
    ImageText,
}

fn mode_key(mode: DisplayMode) -> &'static str {
    match mode {
        DisplayMode::Default => "default",
        DisplayMode::List => "list",
        DisplayMode::Custom => "custom",
    }
}

#[component]
pub fn DisplayTab() -> Element {
    rsx! {
        DisplayEditor { kind: Kind::Music }
        DisplayEditor { kind: Kind::Movies }
        DisplayEditor { kind: Kind::Episodes }
    }
}

#[component]
fn DisplayEditor(kind: Kind) -> Element {
    let state = use_context::<AppState>();
    let mut form = state.form;
    let mut focus = use_signal(|| Line::State);
    let d = kind.get(&form.read()).clone();
    let (show_simple, append_prefix, add_divider) = {
        let f = form.read();
        (f.show_simple, f.append_prefix, f.add_divider)
    };

    let mut insert = move |name: &'static str| {
        let mut form = form.write();
        let d = kind.get_mut(&mut form);

        match d.mode {
            DisplayMode::Default => {}
            DisplayMode::List => {
                if name != "sep" {
                    if !d.list.trim().is_empty() {
                        d.list.push_str(", ");
                    }
                    d.list.push_str(name);
                }
            }
            DisplayMode::Custom => {
                let target = match focus() {
                    Line::Details => &mut d.details,
                    Line::State => &mut d.state,
                    Line::ImageText => &mut d.image_text,
                };
                target.push_str(&format!("{{{name}}}"));
            }
        }
    };

    rsx! {
        Section { title: kind.title(),
            Choice {
                options: vec![
                    ("default".to_string(), "Default".to_string()),
                    ("list".to_string(), "List".to_string()),
                    ("custom".to_string(), "Custom".to_string()),
                ],
                value: mode_key(d.mode).to_string(),
                onchange: move |key: String| {
                    kind.get_mut(&mut form.write()).mode = match key.as_str() {
                        "list" => DisplayMode::List,
                        "custom" => DisplayMode::Custom,
                        _ => DisplayMode::Default,
                    };
                },
            }
            match d.mode {
                DisplayMode::Default => rsx! {
                    Help { text: "Uses the built-in layout." }
                    if kind == Kind::Episodes {
                        Toggle {
                            label: "Hide episode title",
                            checked: show_simple,
                            onchange: move |v| form.write().show_simple = v,
                        }
                        Toggle {
                            label: "Pad season and episode numbers (S01E01)",
                            checked: append_prefix,
                            onchange: move |v| form.write().append_prefix = v,
                        }
                        Toggle {
                            label: "Divider between season and episode (S1 - E1)",
                            checked: add_divider,
                            onchange: move |v| form.write().add_divider = v,
                        }
                    }
                },
                DisplayMode::List => rsx! {
                    TextField {
                        label: "Placeholders",
                        value: d.list.clone(),
                        placeholder: "genres, year",
                        help: "Comma separated placeholders appended to the default state line.",
                        oninput: move |v| kind.get_mut(&mut form.write()).list = v,
                    }
                },
                DisplayMode::Custom => rsx! {
                    TextField {
                        label: "Details (first line)",
                        value: d.details.clone(),
                        onfocus: move |_| focus.set(Line::Details),
                        oninput: move |v| kind.get_mut(&mut form.write()).details = v,
                    }
                    TextField {
                        label: "State (second line)",
                        value: d.state.clone(),
                        onfocus: move |_| focus.set(Line::State),
                        oninput: move |v| kind.get_mut(&mut form.write()).state = v,
                    }
                    TextField {
                        label: "Image text (hover text)",
                        value: d.image_text.clone(),
                        onfocus: move |_| focus.set(Line::ImageText),
                        oninput: move |v| kind.get_mut(&mut form.write()).image_text = v,
                    }
                    Help { text: "{{__default}} inserts the built-in text for a line." }
                },
            }
            if d.mode != DisplayMode::Default {
                div { class: "chips",
                    for name in kind.placeholders() {
                        Suggestion {
                            key: "{name}",
                            text: "{{{name}}}",
                            onclick: move |_| insert(name),
                        }
                    }
                    if d.mode == DisplayMode::Custom {
                        Suggestion {
                            text: "{{__default}}",
                            onclick: move |_| insert("__default"),
                        }
                    }
                }
            }
            TextField {
                label: "Separator",
                value: d.separator.clone(),
                placeholder: "-",
                help: "Replaces {{sep}}.",
                oninput: move |v| kind.get_mut(&mut form.write()).separator = v,
            }
            Choice {
                label: "Member list status shows",
                options: vec![
                    (String::new(), "Name".to_string()),
                    ("state".to_string(), "State".to_string()),
                    ("details".to_string(), "Details".to_string()),
                ],
                value: d.status_display_type.clone(),
                onchange: move |key: String| {
                    kind.get_mut(&mut form.write()).status_display_type = key;
                },
            }
        }
    }
}
