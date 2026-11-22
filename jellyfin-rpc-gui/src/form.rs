//! Flat, UI friendly model of `main.json`, converted to and from `ConfigBuilder`.

use jellyfin_rpc::{
    config::{
        Blacklist, ConfigBuilder, Display, DiscordBuilder, DisplayOptionsBuilder, ImagesBuilder,
        Imgur, JellyfinBuilder, Username, Zipline,
    },
    Button, DisplayFormat, MediaType,
};

pub const DEFAULT_DETAILS: &str = "{__default}";
pub const DEFAULT_STATE: &str = "{__default}";
pub const DEFAULT_IMAGE_TEXT: &str = "Jellyfin-RPC v{version}";

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DisplayMode {
    /// `display` is not set, the built-in default is used.
    #[default]
    Default,
    /// Legacy list of placeholders appended to the default state.
    List,
    /// Fully custom details, state and image text.
    Custom,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DisplayForm {
    pub mode: DisplayMode,
    pub list: String,
    pub details: String,
    pub state: String,
    pub image_text: String,
    pub separator: String,
    /// Empty, `name`, `state` or `details`.
    pub status_display_type: String,
}

impl Default for DisplayForm {
    fn default() -> Self {
        Self {
            mode: DisplayMode::Default,
            list: String::new(),
            details: DEFAULT_DETAILS.to_string(),
            state: DEFAULT_STATE.to_string(),
            image_text: DEFAULT_IMAGE_TEXT.to_string(),
            separator: String::new(),
            status_display_type: String::new(),
        }
    }
}

impl DisplayForm {
    fn from_options(options: &Option<DisplayOptionsBuilder>) -> Self {
        let mut form = Self::default();

        let Some(options) = options else {
            return form;
        };

        match &options.display {
            None => {}
            Some(Display::Vec(items)) => {
                form.mode = DisplayMode::List;
                form.list = items.join(", ");
            }
            Some(Display::String(items)) => {
                form.mode = DisplayMode::List;
                form.list = items.clone();
            }
            Some(Display::CustomFormat(format)) => {
                form.mode = DisplayMode::Custom;
                form.details = format.details_text.clone().unwrap_or_default();
                form.state = format.state_text.clone().unwrap_or_default();
                form.image_text = format.image_text.clone().unwrap_or_default();
            }
        }

        form.separator = options.separator.clone().unwrap_or_default();
        form.status_display_type = options.status_display_type.clone().unwrap_or_default();
        form
    }

    fn to_options(&self) -> Option<DisplayOptionsBuilder> {
        let display = match self.mode {
            DisplayMode::Default => None,
            DisplayMode::List => Some(Display::Vec(
                self.list
                    .split(',')
                    .map(|i| i.trim().trim_matches(|c| c == '{' || c == '}').to_string())
                    .filter(|i| !i.is_empty())
                    .collect(),
            )),
            DisplayMode::Custom => Some(Display::CustomFormat(DisplayFormat {
                details_text: Some(self.details.clone()),
                state_text: Some(self.state.clone()),
                image_text: Some(self.image_text.clone()),
            })),
        };

        let separator = non_empty(&self.separator);
        let status_display_type = non_empty(&self.status_display_type);

        if display.is_none() && separator.is_none() && status_display_type.is_none() {
            return None;
        }

        Some(DisplayOptionsBuilder {
            display,
            separator,
            status_display_type,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonsMode {
    /// Buttons are generated from the external links of the item.
    #[default]
    Auto,
    Custom,
    Disabled,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ButtonForm {
    pub dynamic: bool,
    pub name: String,
    pub url: String,
}

impl Default for ButtonForm {
    fn default() -> Self {
        Self {
            dynamic: true,
            name: String::new(),
            url: String::new(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ImageHost {
    /// The Jellyfin image url is used directly.
    #[default]
    Jellyfin,
    Imgur,
    Litterbox,
    Zipline,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Form {
    pub url: String,
    pub api_key: String,
    pub usernames: Vec<String>,
    pub self_signed_cert: bool,
    pub show_simple: bool,
    pub append_prefix: bool,
    pub add_divider: bool,
    pub blacklist_types: Vec<MediaType>,
    pub blacklist_libraries: Vec<String>,
    pub music: DisplayForm,
    pub movies: DisplayForm,
    pub episodes: DisplayForm,

    pub application_id: String,
    pub show_paused: bool,
    pub buttons_mode: ButtonsMode,
    pub buttons: Vec<ButtonForm>,

    pub imgur_client_id: String,
    pub zipline_url: String,
    pub zipline_token: String,
    pub zipline_expiry: String,
    pub zipline_format: String,
    pub zipline_compression_percent: String,
    pub zipline_compression_type: String,
    pub zipline_original_name: bool,
    pub zipline_folder: String,
    pub zipline_domain: String,

    pub enable_images: bool,
    pub image_host: ImageHost,
    pub process_images: bool,
    pub image_size: String,
    pub image_bg: bool,
    pub image_bg_blur: String,
    pub image_corner_radius: String,
}

impl Default for Form {
    fn default() -> Self {
        Self::from_builder(&ConfigBuilder::default())
    }
}

fn non_empty(s: &str) -> Option<String> {
    let s = s.trim();
    (!s.is_empty()).then(|| s.to_string())
}

fn number_text<T: ToString>(n: Option<T>) -> String {
    n.map(|n| n.to_string()).unwrap_or_default()
}

impl Form {
    pub fn from_builder(b: &ConfigBuilder) -> Self {
        let usernames = match &b.jellyfin.username {
            Username::Vec(v) => v.clone(),
            Username::String(s) => s.split(',').map(|u| u.trim().to_string()).collect(),
        }
        .into_iter()
        .filter(|u| !u.is_empty())
        .collect();

        let blacklist = b.jellyfin.blacklist.clone();
        let discord = b.discord.clone();
        let images = b.images.clone();
        let zipline = b.zipline.clone().unwrap_or_default();

        let (buttons_mode, buttons) = match discord.as_ref().and_then(|d| d.buttons.clone()) {
            None => (ButtonsMode::Auto, Vec::new()),
            Some(buttons) if buttons.is_empty() => (ButtonsMode::Disabled, Vec::new()),
            Some(buttons) => (
                ButtonsMode::Custom,
                buttons
                    .into_iter()
                    .take(2)
                    .map(|b| {
                        if b == Button::default() {
                            ButtonForm::default()
                        } else {
                            ButtonForm {
                                dynamic: false,
                                name: b.name,
                                url: b.url,
                            }
                        }
                    })
                    .collect(),
            ),
        };

        let image_host = match &images {
            Some(i) if i.imgur_images == Some(true) => ImageHost::Imgur,
            Some(i) if i.litterbox_images == Some(true) => ImageHost::Litterbox,
            Some(i) if i.zipline_images == Some(true) => ImageHost::Zipline,
            _ => ImageHost::Jellyfin,
        };

        Self {
            url: b.jellyfin.url.clone(),
            api_key: b.jellyfin.api_key.clone(),
            usernames,
            self_signed_cert: b.jellyfin.self_signed_cert.unwrap_or(false),
            show_simple: b.jellyfin.show_simple.unwrap_or(false),
            append_prefix: b.jellyfin.append_prefix.unwrap_or(false),
            add_divider: b.jellyfin.add_divider.unwrap_or(false),
            blacklist_types: blacklist
                .as_ref()
                .and_then(|b| b.media_types.clone())
                .unwrap_or_default(),
            blacklist_libraries: blacklist
                .as_ref()
                .and_then(|b| b.libraries.clone())
                .unwrap_or_default(),
            music: DisplayForm::from_options(&b.jellyfin.music),
            movies: DisplayForm::from_options(&b.jellyfin.movies),
            episodes: DisplayForm::from_options(&b.jellyfin.episodes),

            application_id: discord
                .as_ref()
                .and_then(|d| d.application_id.clone())
                .unwrap_or_default(),
            show_paused: discord
                .as_ref()
                .and_then(|d| d.show_paused)
                .unwrap_or(true),
            buttons_mode,
            buttons,

            imgur_client_id: b
                .imgur
                .as_ref()
                .and_then(|i| i.client_id.clone())
                .unwrap_or_default(),
            zipline_url: zipline.url.unwrap_or_default(),
            zipline_token: zipline.token.unwrap_or_default(),
            zipline_expiry: zipline.expiry.unwrap_or_default(),
            zipline_format: zipline.format.unwrap_or_default(),
            zipline_compression_percent: number_text(zipline.image_compression_percent),
            zipline_compression_type: zipline.image_compression_type.unwrap_or_default(),
            zipline_original_name: zipline.original_name,
            zipline_folder: zipline.folder.unwrap_or_default(),
            zipline_domain: zipline.domain.unwrap_or_default(),

            enable_images: images
                .as_ref()
                .and_then(|i| i.enable_images)
                .unwrap_or(false),
            image_host,
            process_images: images
                .as_ref()
                .and_then(|i| i.process_images)
                .unwrap_or(true),
            image_size: number_text(images.as_ref().and_then(|i| i.size)),
            image_bg: images.as_ref().and_then(|i| i.bg).unwrap_or(true),
            image_bg_blur: images
                .as_ref()
                .and_then(|i| i.bg_blur)
                .unwrap_or(3.0)
                .to_string(),
            image_corner_radius: images
                .as_ref()
                .and_then(|i| i.corner_radius)
                .unwrap_or(4.0)
                .to_string(),
        }
    }

    pub fn to_builder(&self) -> ConfigBuilder {
        let blacklist = (!self.blacklist_types.is_empty() || !self.blacklist_libraries.is_empty())
            .then(|| Blacklist {
                media_types: (!self.blacklist_types.is_empty()).then(|| self.blacklist_types.clone()),
                libraries: (!self.blacklist_libraries.is_empty())
                    .then(|| self.blacklist_libraries.clone()),
            });

        let buttons = match self.buttons_mode {
            ButtonsMode::Auto => None,
            ButtonsMode::Disabled => Some(Vec::new()),
            ButtonsMode::Custom => Some(
                self.buttons
                    .iter()
                    .take(2)
                    .map(|b| {
                        if b.dynamic {
                            Button::default()
                        } else {
                            Button::new(b.name.clone(), b.url.clone())
                        }
                    })
                    .collect(),
            ),
        };

        let zipline = Zipline {
            url: non_empty(&self.zipline_url),
            token: non_empty(&self.zipline_token),
            expiry: non_empty(&self.zipline_expiry),
            format: non_empty(&self.zipline_format),
            image_compression_percent: self.zipline_compression_percent.trim().parse().ok(),
            image_compression_type: non_empty(&self.zipline_compression_type),
            original_name: self.zipline_original_name,
            folder: non_empty(&self.zipline_folder),
            domain: non_empty(&self.zipline_domain),
        };
        let zipline_used = zipline != Zipline::default();

        ConfigBuilder {
            jellyfin: JellyfinBuilder {
                url: self.url.trim().to_string(),
                api_key: self.api_key.trim().to_string(),
                username: Username::Vec(
                    self.usernames
                        .iter()
                        .map(|u| u.trim().to_string())
                        .filter(|u| !u.is_empty())
                        .collect(),
                ),
                music: self.music.to_options(),
                movies: self.movies.to_options(),
                episodes: self.episodes.to_options(),
                blacklist,
                self_signed_cert: Some(self.self_signed_cert),
                show_simple: Some(self.show_simple),
                append_prefix: Some(self.append_prefix),
                add_divider: Some(self.add_divider),
            },
            discord: Some(DiscordBuilder {
                application_id: non_empty(&self.application_id),
                buttons,
                show_paused: Some(self.show_paused),
            }),
            imgur: non_empty(&self.imgur_client_id).map(|id| Imgur {
                client_id: Some(id),
            }),
            zipline: zipline_used.then_some(zipline),
            images: Some(ImagesBuilder {
                enable_images: Some(self.enable_images),
                imgur_images: Some(self.image_host == ImageHost::Imgur),
                litterbox_images: Some(self.image_host == ImageHost::Litterbox),
                zipline_images: Some(self.image_host == ImageHost::Zipline),
                process_images: Some(self.process_images),
                size: self.image_size.trim().parse().ok(),
                bg: Some(self.image_bg),
                bg_blur: Some(self.image_bg_blur.trim().parse().unwrap_or(3.0)),
                corner_radius: Some(self.image_corner_radius.trim().parse().unwrap_or(4.0)),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_form_roundtrips() {
        let form = Form::default();
        assert_eq!(Form::from_builder(&form.to_builder()), form);
    }

    #[test]
    fn edited_form_roundtrips_through_json() {
        let mut form = Form::default();
        form.url = "https://jf.example.com".into();
        form.api_key = "key".into();
        form.usernames = vec!["a".into(), "b".into()];
        form.blacklist_types = vec![MediaType::Music];
        form.blacklist_libraries = vec!["Anime".into()];
        form.movies.mode = DisplayMode::Custom;
        form.movies.details = "{title}".into();
        form.music.mode = DisplayMode::List;
        form.music.list = "genres, year".into();
        form.music.status_display_type = "details".into();
        form.buttons_mode = ButtonsMode::Custom;
        form.buttons = vec![
            ButtonForm::default(),
            ButtonForm {
                dynamic: false,
                name: "Site".into(),
                url: "https://example.com".into(),
            },
        ];
        form.enable_images = true;
        form.image_host = ImageHost::Zipline;
        form.zipline_url = "https://z.example.com/api/upload".into();
        form.zipline_compression_percent = "80".into();
        form.image_size = "512".into();

        let json = serde_json::to_string(&form.to_builder()).unwrap();
        let parsed: ConfigBuilder = serde_json::from_str(&json).unwrap();
        assert_eq!(Form::from_builder(&parsed), form);
    }

    #[test]
    fn disabled_buttons_are_kept_empty() {
        let mut form = Form::default();
        form.buttons_mode = ButtonsMode::Disabled;
        let builder = form.to_builder();
        assert_eq!(builder.discord.unwrap().buttons, Some(Vec::new()));
    }
}
