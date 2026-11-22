use dioxus::prelude::*;

use crate::{
    app::AppState,
    form::{ButtonForm, ButtonsMode},
    ui::widgets::{Choice, Help, Section, TextField, Toggle},
};

fn mode_key(mode: ButtonsMode) -> &'static str {
    match mode {
        ButtonsMode::Auto => "auto",
        ButtonsMode::Custom => "custom",
        ButtonsMode::Disabled => "disabled",
    }
}

#[component]
pub fn DiscordTab() -> Element {
    let state = use_context::<AppState>();
    let mut form = state.form;
    let f = form.read().clone();

    rsx! {
        Section { title: "Application",
            TextField {
                label: "Application ID",
                value: f.application_id.clone(),
                placeholder: "1053747938519679018",
                help: "Discord application whose name is shown as the activity name. Leave empty for the default.",
                oninput: move |v| form.write().application_id = v,
            }
            Toggle {
                label: "Show activity while paused",
                checked: f.show_paused,
                help: "When off, the activity is cleared as soon as playback is paused.",
                onchange: move |v| form.write().show_paused = v,
            }
        }
        Section { title: "Buttons",
            Choice {
                options: vec![
                    ("auto".to_string(), "From item links".to_string()),
                    ("custom".to_string(), "Custom".to_string()),
                    ("disabled".to_string(), "None".to_string()),
                ],
                value: mode_key(f.buttons_mode).to_string(),
                onchange: move |key: String| {
                    let mut form = form.write();
                    form.buttons_mode = match key.as_str() {
                        "custom" => ButtonsMode::Custom,
                        "disabled" => ButtonsMode::Disabled,
                        _ => ButtonsMode::Auto,
                    };
                    if form.buttons_mode == ButtonsMode::Custom && form.buttons.is_empty() {
                        form.buttons.push(ButtonForm::default());
                    }
                },
            }
            match f.buttons_mode {
                ButtonsMode::Auto => rsx! {
                    Help { text: "Buttons are generated from the external links of the item being played (e.g. IMDb, TMDb)." }
                },
                ButtonsMode::Disabled => rsx! {
                    Help { text: "No buttons are shown." }
                },
                ButtonsMode::Custom => rsx! {
                    Help { text: "Up to two buttons. A dynamic button is filled with the next external link of the item." }
                    for (i, button) in f.buttons.iter().cloned().enumerate() {
                        div { key: "{i}", class: "button-editor",
                            div { class: "row between",
                                strong { "Button {i + 1}" }
                                button {
                                    class: "btn small",
                                    onclick: move |_| { form.write().buttons.remove(i); },
                                    "Remove"
                                }
                            }
                            Toggle {
                                label: "Dynamic (from item links)",
                                checked: button.dynamic,
                                onchange: move |v| form.write().buttons[i].dynamic = v,
                            }
                            if !button.dynamic {
                                TextField {
                                    label: "Label",
                                    value: button.name.clone(),
                                    oninput: move |v| form.write().buttons[i].name = v,
                                }
                                TextField {
                                    label: "URL",
                                    value: button.url.clone(),
                                    placeholder: "https://example.com",
                                    oninput: move |v| form.write().buttons[i].url = v,
                                }
                            }
                        }
                    }
                    if f.buttons.len() < 2 {
                        button {
                            class: "btn",
                            onclick: move |_| form.write().buttons.push(ButtonForm::default()),
                            "Add button"
                        }
                    }
                },
            }
        }
    }
}
