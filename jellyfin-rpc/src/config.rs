use crate::{
    Button, ClientBuilder, DisplayFormat, EpisodeDisplayOptions, MediaType, StatusType, VERSION,
};
use log::debug;
use serde::{Deserialize, Serialize};
use std::env;

/// Main struct containing every other struct in the file.
///
/// The config file is parsed into this struct.
pub struct Config {
    /// Jellyfin configuration.
    ///
    /// Has every required part of the config, hence why its not an `Option<Jellyfin>`.
    pub jellyfin: Jellyfin,
    /// Discord configuration.
    pub discord: Discord,
    /// Imgur configuration.
    pub imgur: Imgur,
    /// Zipline configuration.
    pub zipline: Zipline,
    /// Images configuration.
    pub images: Images,
}

/// This struct contains every "required" part of the config.
pub struct Jellyfin {
    /// URL to the jellyfin server.
    pub url: String,
    /// Api key from the jellyfin server, used to gather what's being watched.
    pub api_key: String,
    /// Username of the person that info should be gathered from.
    pub username: Vec<String>,
    /// Contains configuration for Music display.
    pub music: DisplayOptions,
    /// Contains configuration for Movie display.
    pub movies: DisplayOptions,
    /// Contains configuration for Episode display.
    pub episodes: DisplayOptions,
    /// Blacklist configuration.
    pub blacklist: Blacklist,
    /// Self signed certificate option
    pub self_signed_cert: bool,
    /// Simple episode name
    pub show_simple: bool,
    /// Add "0" before season/episode number if lower than 10.
    pub append_prefix: bool,
    /// Add a divider between numbers
    pub add_divider: bool,
}

/// Contains configuration for Music/Movie display.
pub struct DisplayOptions {
    /// Display is where you tell the program what should be displayed.
    pub display: Option<DisplayFormat>,
    /// Separator is what should be between the artist(s) and the `display` options.
    pub separator: Option<String>,
    /// Whether the to display the name, state, or details in the status title.
    pub status_display_type: Option<StatusType>,
}

/// Discord configuration
pub struct Discord {
    /// Set a custom Application ID to be used.
    pub application_id: Option<String>,
    /// Set custom buttons to be displayed.
    pub buttons: Option<Vec<Button>>,
    /// Show status when media is paused
    pub show_paused: bool,
}

/// Images configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Images {
    /// Enables images, not everyone wants them so its a toggle.
    pub enable_images: bool,
    /// Enables imgur images.
    pub imgur_images: bool,
    /// Enables litterbox images.
    pub litterbox_images: bool,
    /// Enables Zipline images.
    pub zipline_images: bool,
    /// Processes images by making them square and adding a blur.
    pub process_images: bool,
    /// The size of the output square image canvas (e.g., 512 for 512x512px).
    pub size: Option<u32>,
    /// Whether to create a blurred background. Default: true.
    pub bg: bool,
    /// The blur radius for the background image as a percentage of canvas size.
    /// Default: 3.
    pub bg_blur: f32,
    /// Corner radius as a percentage of the image size.
    /// Only applied when background is disabled. Default: 4.
    pub corner_radius: Option<f32>,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct ConfigBuilder {
    pub jellyfin: JellyfinBuilder,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discord: Option<DiscordBuilder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imgur: Option<Imgur>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zipline: Option<Zipline>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub images: Option<ImagesBuilder>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JellyfinBuilder {
    pub url: String,
    pub api_key: String,
    pub username: Username,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<DisplayOptionsBuilder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub movies: Option<DisplayOptionsBuilder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episodes: Option<DisplayOptionsBuilder>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blacklist: Option<Blacklist>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_signed_cert: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_simple: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub append_prefix: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub add_divider: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Username {
    /// If the username is a `Vec<String>`.
    Vec(Vec<String>),
    /// If the username is a `String`.
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DisplayOptionsBuilder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<Display>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub separator: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_display_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum Display {
    /// If the Display is a `Vec<String>`.
    Vec(Vec<String>),
    /// If the Display is a comma separated `String`.
    String(String),
    /// If the Display is a `DisplayFormat` struct.
    CustomFormat(DisplayFormat),
}

/// Blacklist MediaTypes and libraries.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Blacklist {
    /// `Vec<String>` of MediaTypes to blacklist
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_types: Option<Vec<MediaType>>,
    /// `Vec<String>` of libraries to blacklist
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub libraries: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct DiscordBuilder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<Button>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub show_paused: Option<bool>,
}

/// Imgur configuration
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Imgur {
    /// Contains the client ID used to upload images to imgur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct Zipline {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_compression_percent: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_compression_type: Option<String>,
    #[serde(default)]
    pub original_name: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ImagesBuilder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_images: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imgur_images: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub litterbox_images: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zipline_images: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_images: Option<bool>,
    /// The size of the output square image canvas (e.g., 512 for 512x512px).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    /// Whether to create a blurred background. Default: true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg: Option<bool>,
    /// The blur radius for the background image as a percentage of canvas size.
    /// Default: 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_blur: Option<f32>,
    /// Corner radius as a percentage of the image size.
    /// Only applied when background is disabled. Default: 4.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f32>,
}

/// Find urls.json in filesystem, used to store images that were already previously uploaded to imgur.
///
/// This is to avoid the user having to specify a filepath on launch.
///
/// Default urls.json path depends on OS
/// Windows: `%appdata%\jellyfin-rpc\urls.json`
/// Linux/macOS: `~/.config/jellyfin-rpc/urls.json`
pub fn get_urls_path() -> Result<String, Box<dyn std::error::Error>> {
    if cfg!(not(windows)) {
        debug!("Platform is not Windows");
        let xdg_config_home = match env::var("XDG_CONFIG_HOME") {
            Ok(xdg_config_home) => xdg_config_home,
            Err(_) => env::var("HOME")? + "/.config",
        };

        Ok(xdg_config_home + ("/jellyfin-rpc/urls.json"))
    } else {
        debug!("Platform is Windows");
        let app_data = env::var("APPDATA")?;
        Ok(app_data + r"\jellyfin-rpc\urls.json")
    }
}

/// Find default config path (main.json) in filesystem.
///
/// This is to avoid the user having to specify a filepath on launch.
///
/// Default config path depends on OS
/// Windows: `%appdata%\jellyfin-rpc\main.json`
/// Linux/macOS: `~/.config/jellyfin-rpc/main.json`
pub fn get_config_path() -> Result<String, Box<dyn std::error::Error>> {
    debug!("Getting config path");
    if cfg!(not(windows)) {
        debug!("Platform is not Windows");
        let xdg_config_home = match env::var("XDG_CONFIG_HOME") {
            Ok(xdg_config_home) => xdg_config_home,
            Err(_) => env::var("HOME")? + "/.config",
        };

        Ok(xdg_config_home + "/jellyfin-rpc/main.json")
    } else {
        debug!("Platform is Windows");
        let app_data = env::var("APPDATA")?;
        Ok(app_data + r"\jellyfin-rpc\main.json")
    }
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            jellyfin: JellyfinBuilder {
                url: "".to_string(),
                username: Username::String("".to_string()),
                api_key: "".to_string(),
                music: None,
                movies: None,
                episodes: None,
                blacklist: None,
                self_signed_cert: None,
                show_simple: Some(false),
                append_prefix: Some(false),
                add_divider: Some(false),
            },
            discord: None,
            imgur: None,
            zipline: None,
            images: None,
        }
    }

    /// Writes the config as pretty JSON to the given path, creating parent directories if needed.
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    /// Loads the config from the given path.
    pub fn load(self, path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        debug!("Config path is: {}", path);

        let data = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&data)?;

        debug!("Config loaded successfully");

        Ok(config)
    }

    pub fn build(self) -> Config {
        let username = match self.jellyfin.username {
            Username::Vec(usernames) => usernames,
            Username::String(username) => username.split(',').map(|u| u.to_string()).collect(),
        };

        let music_display;
        let music_separator;
        let music_status_display_type;

        if let Some(music) = self.jellyfin.music {
            if let Some(disp) = music.display {
                music_display = Some(match disp {
                    Display::Vec(display) => DisplayFormat::from(display),
                    Display::String(display) => DisplayFormat::from(display),
                    Display::CustomFormat(display) => display,
                });
            } else {
                music_display = None;
            }

            music_separator = music.separator;

            music_status_display_type = music
                .status_display_type
                .and_then(|x| StatusType::try_from(x).ok());
        } else {
            music_display = None;
            music_separator = None;
            music_status_display_type = None;
        }

        let movie_display;
        let movie_separator;
        let movie_status_display_type;

        if let Some(movies) = self.jellyfin.movies {
            if let Some(disp) = movies.display {
                movie_display = Some(match disp {
                    Display::Vec(display) => DisplayFormat::from(display),
                    Display::String(display) => DisplayFormat::from(display),
                    Display::CustomFormat(display) => display,
                });
            } else {
                movie_display = None;
            }

            movie_separator = movies.separator;

            movie_status_display_type = movies
                .status_display_type
                .and_then(|x| StatusType::try_from(x).ok());
        } else {
            movie_display = None;
            movie_separator = None;
            movie_status_display_type = None;
        }

        let episode_display;
        let episode_separator;
        let episode_status_display_type;

        if let Some(episodes) = self.jellyfin.episodes {
            if let Some(disp) = episodes.display {
                episode_display = Some(match disp {
                    Display::Vec(display) => DisplayFormat::from(display),
                    Display::String(display) => DisplayFormat::from(display),
                    Display::CustomFormat(display) => display,
                });
            } else {
                episode_display = None;
            }

            episode_separator = episodes.separator;

            episode_status_display_type = episodes
                .status_display_type
                .and_then(|x| StatusType::try_from(x).ok());
        } else {
            episode_display = None;
            episode_separator = None;
            episode_status_display_type = None;
        }

        let media_types;
        let libraries;

        if let Some(blacklist) = self.jellyfin.blacklist {
            media_types = blacklist.media_types;
            libraries = blacklist.libraries;
        } else {
            media_types = None;
            libraries = None;
        }

        let application_id;
        let buttons;
        let show_paused;

        if let Some(discord) = self.discord {
            application_id = discord.application_id;
            buttons = discord.buttons;
            show_paused = discord.show_paused.unwrap_or(true)
        } else {
            application_id = None;
            buttons = None;
            show_paused = true;
        }

        let client_id;

        if let Some(imgur) = self.imgur {
            client_id = imgur.client_id;
        } else {
            client_id = None
        }

        let zipline = self.zipline.unwrap_or_default();
        
        let enable_images;
        let imgur_images;
        let litterbox_images;
        let zipline_images;
        let process_images;
        let image_size;
        let image_bg;
        let image_bg_blur;
        let image_corner_radius;

        if let Some(images) = self.images {
            enable_images = images.enable_images.unwrap_or(false);
            imgur_images = images.imgur_images.unwrap_or(false);
            litterbox_images = images.litterbox_images.unwrap_or(false);
            zipline_images = images.zipline_images.unwrap_or(false);
            process_images = images.process_images.unwrap_or(true);
            image_size = images.size;
            image_bg = images.bg.unwrap_or(true);
            image_bg_blur = images.bg_blur.unwrap_or(3.0);
            image_corner_radius = images.corner_radius.or(Some(4.0));
        } else {
            enable_images = false;
            imgur_images = false;
            litterbox_images = false;
            zipline_images = false;
            process_images = true;
            image_size = None;
            image_bg = true;
            image_bg_blur = 3.0;
            image_corner_radius = Some(4.0);
        }

        let url;

        if self.jellyfin.url.ends_with("/") {
            url = self.jellyfin.url;
        } else {
            url = self.jellyfin.url + "/"
        }

        Config {
            jellyfin: Jellyfin {
                url,
                api_key: self.jellyfin.api_key,
                username,
                music: DisplayOptions {
                    display: music_display,
                    separator: music_separator,
                    status_display_type: music_status_display_type,
                },
                movies: DisplayOptions {
                    display: movie_display,
                    separator: movie_separator,
                    status_display_type: movie_status_display_type,
                },
                episodes: DisplayOptions {
                    display: episode_display,
                    separator: episode_separator,
                    status_display_type: episode_status_display_type,
                },
                blacklist: Blacklist {
                    media_types,
                    libraries,
                },
                self_signed_cert: self.jellyfin.self_signed_cert.unwrap_or(false),
                show_simple: self.jellyfin.show_simple.unwrap_or(false),
                append_prefix: self.jellyfin.append_prefix.unwrap_or(false),
                add_divider: self.jellyfin.add_divider.unwrap_or(false),
            },
            discord: Discord {
                application_id,
                buttons,
                show_paused,
            },
            imgur: Imgur { client_id },
            zipline,
            images: Images {
                enable_images,
                imgur_images,
                litterbox_images,
                zipline_images,
                process_images,
                size: image_size,
                bg: image_bg,
                bg_blur: image_bg_blur,
                corner_radius: image_corner_radius,
            },
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Creates a `ClientBuilder` from a parsed `Config`.
    ///
    /// `urls_path` is the file used to cache uploaded image urls.
    pub fn from_config(conf: Config, urls_path: &str) -> Self {
        let mut builder = ClientBuilder::new();

        builder
            .api_key(conf.jellyfin.api_key)
            .url(conf.jellyfin.url)
            .usernames(conf.jellyfin.username)
            .self_signed(conf.jellyfin.self_signed_cert)
            .episode_simple(conf.jellyfin.show_simple)
            .episode_divider(conf.jellyfin.add_divider)
            .episode_prefix(conf.jellyfin.append_prefix)
            .show_paused(conf.discord.show_paused)
            .show_images(conf.images.enable_images)
            .use_imgur(conf.images.imgur_images)
            .use_litterbox(conf.images.litterbox_images)
            .use_zipline(conf.images.zipline_images)
            .zipline_url(conf.zipline.url.unwrap_or_default())
            .zipline_token(conf.zipline.token.unwrap_or_default())
            .zipline_expiry(conf.zipline.expiry)
            .zipline_format(conf.zipline.format)
            .zipline_image_compression_percent(conf.zipline.image_compression_percent)
            .zipline_image_compression_type(conf.zipline.image_compression_type)
            .zipline_original_name(conf.zipline.original_name)
            .zipline_folder(conf.zipline.folder)
            .zipline_domain(conf.zipline.domain)
            .process_images(conf.images.process_images)
            .image_size(conf.images.size)
            .image_background(conf.images.bg)
            .image_background_blur(conf.images.bg_blur)
            .image_corner_radius(conf.images.corner_radius)
            .large_image_text(format!("Jellyfin-RPC v{}", VERSION.unwrap_or("UNKNOWN")))
            .imgur_urls_file_location(urls_path)
            .litterbox_urls_file_location(urls_path)
            .zipline_urls_file_location(urls_path);

        if let Some(display) = conf.jellyfin.music.display {
            builder.music_display(display);
        }
        if let Some(separator) = conf.jellyfin.music.separator {
            builder.music_separator(separator);
        }
        if let Some(t) = conf.jellyfin.music.status_display_type {
            builder.music_status_display_type(t);
        }
        if let Some(display) = conf.jellyfin.movies.display {
            builder.movies_display(display);
        }
        if let Some(separator) = conf.jellyfin.movies.separator {
            builder.movies_separator(separator);
        }
        if let Some(t) = conf.jellyfin.movies.status_display_type {
            builder.movies_status_display_type(t);
        }

        if let Some(display) = conf.jellyfin.episodes.display {
            builder.episodes_display(display);
        } else {
            builder.episodes_display(DisplayFormat::from(EpisodeDisplayOptions {
                divider: conf.jellyfin.add_divider,
                prefix: conf.jellyfin.append_prefix,
                simple: conf.jellyfin.show_simple,
            }));
        }
        if let Some(separator) = conf.jellyfin.episodes.separator {
            builder.episodes_separator(separator);
        }
        if let Some(t) = conf.jellyfin.episodes.status_display_type {
            builder.episodes_status_display_type(t);
        }

        if let Some(media_types) = conf.jellyfin.blacklist.media_types {
            builder.blacklist_media_types(media_types);
        }
        if let Some(libraries) = conf.jellyfin.blacklist.libraries {
            builder.blacklist_libraries(libraries);
        }
        if let Some(application_id) = conf.discord.application_id {
            builder.client_id(application_id);
        }
        if let Some(buttons) = conf.discord.buttons {
            builder.buttons(buttons);
        }
        if let Some(client_id) = conf.imgur.client_id {
            builder.imgur_client_id(client_id);
        }

        builder
    }
}
