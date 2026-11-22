use clap::Parser;
use colored::Colorize;
use config::{get_config_path, get_urls_path, Config};
use jellyfin_rpc::{ClientBuilder, VERSION};
use log::{debug, error, info};
use retry::retry_with_index;
use simple_logger::SimpleLogger;
use std::{thread::sleep, time::Duration};
use time::macros::format_description;
mod config;
#[cfg(feature = "updates")]
mod updates;

/*
    TODO: Comments
*/

#[derive(Parser)]
#[command(author = "Radical <Radiicall> <radical@radical.fun>, LeviSnoot <hello@levi.land>")]
#[command(version)]
#[command(about = "Rich presence for Jellyfin", long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", help = "Path to the config file")]
    config: Option<String>,
    #[arg(
        short = 'i',
        long = "image-urls-file",
        help = "Path to image urls file for imgur"
    )]
    image_urls: Option<String>,
    #[arg(
        short = 't',
        long = "wait-time",
        help = "Time to wait between loops in seconds",
        default_value_t = 7
    )]
    wait_time: usize,
    #[arg(
        short = 'v',
        long = "log-level",
        help = "Sets the log level to one of: trace, debug, info, warn, error, off",
        default_value_t = String::from("info")
    )]
    log_level: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", args.log_level);
    }

    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .with_timestamp_format(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .init()
        .unwrap();

    info!("Initializing Jellyfin-RPC");

    #[cfg(feature = "updates")]
    updates::checker();

    let conf_path = &args
        .config
        .unwrap_or(get_config_path().expect("default config path couldn't be determined"));

    let conf = match Config::builder().load(conf_path) {
        Ok(file) => file.build(),
        Err(error) => {
            error!(
                "Config file could not be loaded at path: {}",
                conf_path.red()
            );
            error!("{}", error);
            error!(
                "Please create a proper config file: {}",
                "https://github.com/LeviSnoot/jellyfin-rpc/blob/main/main.schema.json".green()
            );
            std::process::exit(1)
        }
    };

    debug!("Creating jellyfin-rpc client builder");
    let builder = ClientBuilder::from_config(conf, &args.image_urls.unwrap_or(get_urls_path()?));

    debug!("Building client");
    let mut client = builder.build()?;

    info!("Connecting to Discord");
    retry_with_index(
        retry::delay::Exponential::from_millis(1000),
        |current_try| {
            info!("Attempt {}: Trying to connect", current_try);
            match client.connect() {
                Ok(_) => retry::OperationResult::Ok(()),
                Err(err) => {
                    error!("{}", err);
                    retry::OperationResult::Retry(())
                }
            }
        },
    )
    .unwrap();
    info!("Connected!");

    let mut currently_playing = String::new();

    loop {
        sleep(Duration::from_secs(args.wait_time as u64));

        match client.set_activity() {
            Ok(activity) => {
                if activity.is_empty() && !currently_playing.is_empty() {
                    let _ = client.clear_activity();
                    info!("Cleared activity");
                    currently_playing = activity;
                } else if activity != currently_playing {
                    currently_playing = activity;

                    info!("{}", currently_playing);
                }
            }
            Err(err) => {
                // TODO: There has to be a better way to check this..
                if err.to_string() == "content is blacklisted" {
                    debug!("{}", err);
                    continue;
                }

                error!("{}", err);
                debug!("{:?}", err);
                retry_with_index(
                    retry::delay::Exponential::from_millis(1000),
                    |current_try| {
                        info!("Attempt {}: Trying to reconnect", current_try);
                        match client.reconnect() {
                            Ok(_) => retry::OperationResult::Ok(()),
                            Err(err) => {
                                error!("{}", err);
                                retry::OperationResult::Retry(())
                            }
                        }
                    },
                )
                .unwrap();
                info!("Reconnected!");
            }
        }
    }
}
