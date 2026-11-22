use dioxus::prelude::*;
use jellyfin_rpc::MediaType;

use crate::{
    app::AppState,
    ui::widgets::{Section, StringList, TextField, Toggle},
    util::{blocking, client_for},
};

const BLACKLISTABLE: [(MediaType, &str); 6] = [
    (MediaType::Movie, "Movies"),
    (MediaType::Episode, "Episodes"),
    (MediaType::Music, "Music"),
    (MediaType::LiveTv, "Live TV"),
    (MediaType::Book, "Books"),
    (MediaType::AudioBook, "Audiobooks"),
];

#[component]
pub fn JellyfinTab() -> Element {
    let mut state = use_context::<AppState>();
    let mut form = state.form;
    let mut result = use_signal(|| None::<Result<String, String>>);
    let mut testing = use_signal(|| false);

    let test = move |_| async move {
        testing.set(true);
        let f = form.read().clone();

        let out = blocking(move || {
            let client = client_for(&f)?;
            let sessions = client.test_connection().map_err(|e| e.to_string())?;
            let users = client.list_users().unwrap_or_default();
            let libraries = client.list_libraries().unwrap_or_default();
            Ok::<_, String>((sessions, users, libraries))
        })
        .await;

        match out {
            Ok((sessions, users, libraries)) => {
                result.set(Some(Ok(format!(
                    "Connected. {} user(s), {} librar(ies), {sessions} active session(s).",
                    users.len(),
                    libraries.len()
                ))));
                state.users.set(users);
                state.libraries.set(libraries);
            }
            Err(err) => result.set(Some(Err(err))),
        }

        testing.set(false);
    };

    let f = form.read().clone();

    rsx! {
        Section { title: "Connection",
            TextField {
                label: "Server URL",
                value: f.url.clone(),
                placeholder: "https://jellyfin.example.com",
                oninput: move |v| form.write().url = v,
            }
            TextField {
                label: "API key",
                value: f.api_key.clone(),
                secret: true,
                help: "Create one in the Jellyfin dashboard under API Keys.",
                oninput: move |v| form.write().api_key = v,
            }
            Toggle {
                label: "Accept self-signed certificates",
                checked: f.self_signed_cert,
                onchange: move |v| form.write().self_signed_cert = v,
            }
            div { class: "row",
                button {
                    class: "btn",
                    disabled: testing(),
                    onclick: test,
                    if testing() { "Testing..." } else { "Test connection" }
                }
                match result() {
                    Some(Ok(msg)) => rsx! { span { class: "result ok", "{msg}" } },
                    Some(Err(msg)) => rsx! { span { class: "result error", "{msg}" } },
                    None => rsx! {},
                }
            }
        }
        Section { title: "Users",
            StringList {
                label: "Usernames",
                items: f.usernames.clone(),
                placeholder: "Add a username",
                suggestions: (state.users)(),
                help: "Sessions are read from these users, in order of priority. Use Test connection to load suggestions.",
                onchange: move |v| form.write().usernames = v,
            }
        }
        Section { title: "Blacklist",
            div { class: "field",
                span { class: "label", "Media types that are never shown" }
                div { class: "checks",
                    for (media_type, label) in BLACKLISTABLE {
                        Toggle {
                            key: "{label}",
                            label: label,
                            checked: f.blacklist_types.contains(&media_type),
                            onchange: move |on| {
                                let mut form = form.write();
                                form.blacklist_types.retain(|t| *t != media_type);
                                if on {
                                    form.blacklist_types.push(media_type);
                                }
                            },
                        }
                    }
                }
            }
            StringList {
                label: "Libraries that are never shown",
                items: f.blacklist_libraries.clone(),
                placeholder: "Add a library name",
                suggestions: (state.libraries)(),
                onchange: move |v| form.write().blacklist_libraries = v,
            }
        }
    }
}
