use clap::Parser;
use dioxus_desktop::{
    tao::window::{Icon, WindowBuilder},
    Config, LogicalSize, WindowCloseBehaviour,
};
use jellyfin_rpc::config::{get_config_path, get_urls_path, ConfigBuilder};
use std::str::FromStr;

mod agent;
mod app;
mod form;
mod tray;
mod ui;
mod util;

#[derive(Parser)]
#[command(version)]
#[command(about = "Tray agent and GUI for Jellyfin-RPC", long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", help = "Path to the config file")]
    config: Option<String>,
    #[arg(long, help = "Open the window on launch instead of starting in the tray")]
    show: bool,
    #[arg(
        short = 'v',
        long = "log-level",
        help = "Sets the log level to one of: trace, debug, info, warn, error, off",
        default_value_t = String::from("info")
    )]
    log_level: String,
}

/// Everything the UI needs from startup, provided as root context.
#[derive(Clone)]
pub struct Startup {
    pub builder: ConfigBuilder,
    pub load_error: Option<String>,
    pub config_path: String,
    pub agent: agent::AgentHandle,
}

fn main() {
    let args = Args::parse();

    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::from_str(&args.log_level).unwrap_or(log::LevelFilter::Info))
        .init()
        .unwrap();

    let config_path = args
        .config
        .unwrap_or_else(|| get_config_path().expect("default config path couldn't be determined"));

    let (builder, load_error) = if std::path::Path::new(&config_path).exists() {
        match ConfigBuilder::new().load(&config_path) {
            Ok(builder) => (builder, None),
            Err(err) => (ConfigBuilder::new(), Some(err.to_string())),
        }
    } else {
        (ConfigBuilder::new(), None)
    };

    let agent = agent::spawn(builder.clone(), get_urls_path().unwrap_or_default());

    let startup = Startup {
        builder,
        load_error,
        config_path,
        agent,
    };

    let window = WindowBuilder::new()
        .with_title("Jellyfin-RPC")
        .with_visible(args.show)
        .with_inner_size(LogicalSize::new(1200.0, 820.0))
        .with_window_icon(Icon::from_rgba(tray::icon_rgba(64), 64, 64).ok());

    let config = Config::new()
        .with_window(window)
        .with_menu(None::<dioxus_desktop::muda::Menu>)
        .with_close_behaviour(WindowCloseBehaviour::WindowHides)
        .with_exits_when_last_window_closes(false);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .with_context(startup)
        .launch(app::App);
}
