use dioxus::prelude::*;

use crate::{
    app::AppState,
    form::ImageHost,
    ui::widgets::{Choice, Help, Section, TextField, Toggle},
};

fn host_key(host: ImageHost) -> &'static str {
    match host {
        ImageHost::Jellyfin => "jellyfin",
        ImageHost::Imgur => "imgur",
        ImageHost::Litterbox => "litterbox",
        ImageHost::Zipline => "zipline",
    }
}

#[component]
pub fn ImagesTab() -> Element {
    let state = use_context::<AppState>();
    let mut form = state.form;
    let f = form.read().clone();

    rsx! {
        Section { title: "Cover images",
            Toggle {
                label: "Show cover images",
                checked: f.enable_images,
                onchange: move |v| form.write().enable_images = v,
            }
            if f.enable_images {
                div { class: "field",
                    Choice {
                        label: "Image host",
                        options: vec![
                            ("jellyfin".to_string(), "Jellyfin (direct)".to_string()),
                            ("imgur".to_string(), "Imgur".to_string()),
                            ("litterbox".to_string(), "Litterbox".to_string()),
                            ("zipline".to_string(), "Zipline".to_string()),
                        ],
                        value: host_key(f.image_host).to_string(),
                        onchange: move |key: String| {
                            form.write().image_host = match key.as_str() {
                                "imgur" => ImageHost::Imgur,
                                "litterbox" => ImageHost::Litterbox,
                                "zipline" => ImageHost::Zipline,
                                _ => ImageHost::Jellyfin,
                            };
                        },
                    }
                    match f.image_host {
                        ImageHost::Jellyfin => rsx! { Help { text: "Discord loads the image straight from your Jellyfin server, so it must be reachable from the internet. Processing is not applied." } },
                        ImageHost::Imgur => rsx! { Help { text: "Images are uploaded to Imgur and the links are cached." } },
                        ImageHost::Litterbox => rsx! { Help { text: "Images are uploaded to litterbox.catbox.moe and expire after 72 hours; they are re-uploaded automatically." } },
                        ImageHost::Zipline => rsx! { Help { text: "Images are uploaded to your own Zipline instance." } },
                    }
                }
            }
        }
        if f.enable_images && f.image_host == ImageHost::Imgur {
            Section { title: "Imgur",
                TextField {
                    label: "Client ID",
                    value: f.imgur_client_id.clone(),
                    secret: true,
                    oninput: move |v| form.write().imgur_client_id = v,
                }
            }
        }
        if f.enable_images && f.image_host == ImageHost::Zipline {
            Section { title: "Zipline",
                TextField {
                    label: "Upload URL",
                    value: f.zipline_url.clone(),
                    placeholder: "https://zipline.example.com/api/upload",
                    oninput: move |v| form.write().zipline_url = v,
                }
                TextField {
                    label: "Token",
                    value: f.zipline_token.clone(),
                    secret: true,
                    oninput: move |v| form.write().zipline_token = v,
                }
                TextField {
                    label: "Expiry",
                    value: f.zipline_expiry.clone(),
                    placeholder: "3d",
                    help: "Relative (s, m, h, d, w) such as 3d, or date=<RFC 3339 timestamp>.",
                    oninput: move |v| form.write().zipline_expiry = v,
                }
                TextField {
                    label: "Name format",
                    value: f.zipline_format.clone(),
                    oninput: move |v| form.write().zipline_format = v,
                }
                TextField {
                    label: "Compression percent",
                    value: f.zipline_compression_percent.clone(),
                    number: true,
                    oninput: move |v| form.write().zipline_compression_percent = v,
                }
                TextField {
                    label: "Compression type",
                    value: f.zipline_compression_type.clone(),
                    oninput: move |v| form.write().zipline_compression_type = v,
                }
                TextField {
                    label: "Folder",
                    value: f.zipline_folder.clone(),
                    oninput: move |v| form.write().zipline_folder = v,
                }
                TextField {
                    label: "Domain",
                    value: f.zipline_domain.clone(),
                    oninput: move |v| form.write().zipline_domain = v,
                }
                Toggle {
                    label: "Keep original file name",
                    checked: f.zipline_original_name,
                    onchange: move |v| form.write().zipline_original_name = v,
                }
            }
        }
        if f.enable_images && f.image_host != ImageHost::Jellyfin {
            Section { title: "Image processing",
                Toggle {
                    label: "Make images square",
                    checked: f.process_images,
                    help: "Processes the cover before uploading so it fits Discord's square artwork.",
                    onchange: move |v| form.write().process_images = v,
                }
                if f.process_images {
                    TextField {
                        label: "Size (px)",
                        value: f.image_size.clone(),
                        number: true,
                        placeholder: "Original size",
                        oninput: move |v| form.write().image_size = v,
                    }
                    Toggle {
                        label: "Blurred background",
                        checked: f.image_bg,
                        onchange: move |v| form.write().image_bg = v,
                    }
                    if f.image_bg {
                        TextField {
                            label: "Blur (% of size, 0-50)",
                            value: f.image_bg_blur.clone(),
                            number: true,
                            oninput: move |v| form.write().image_bg_blur = v,
                        }
                    } else {
                        TextField {
                            label: "Corner radius (% of size, 0-50)",
                            value: f.image_corner_radius.clone(),
                            number: true,
                            oninput: move |v| form.write().image_corner_radius = v,
                        }
                    }
                }
            }
        }
    }
}
