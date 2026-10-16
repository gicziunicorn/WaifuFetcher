#![allow(unused)]
#![warn(unused_extern_crates)]
mod app;
mod config;
// mod fetcher;
// mod sites;
pub mod gui;
pub mod components;

use std::{fs, net::Ipv4Addr};
use dirs::{data_dir, data_local_dir, home_dir};
use iced::Theme;
use time::{UtcOffset, format_description, macros::format_description as timefmt};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, Layer, fmt::{self, time::{OffsetTime, time}}, layer::SubscriberExt, util::SubscriberInitExt};
use waifu_api::{fetcher::Fetcher, site::Sites};
use wreq::Client;
use log::{debug, error, info, warn};

use app::App;


static APP_NAME: &str = "WaifuFetcher";

// Create a user agent that can be sent to the API.
// Most sites require or at least recommend this.
static APP_USER_AGENT: &str = "WaifuFetcher (https://github.com/gicziunicorn/WaifuFetcher)";


// Make converting errors to String easier
trait ResultToString<T> {
    fn err_to_string(self) -> Result<T, String>;
}
impl<T, E: std::fmt::Display> ResultToString<T> for Result<T, E> {
    fn err_to_string(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

trait ImagePath<T> {
    fn image_path(&self) -> String;
}
impl<T, S: ToString> ImagePath<T> for S {
    fn image_path(&self) -> String {
        // Nekos.moe -> nekos-moe
        format!("{}.ico", self.to_string().to_lowercase())
    }
}



pub fn main() {
    if let Err(e) = try_main() {
        error!("An error occurred: {}", e);
        std::process::exit(1);
    }
}

// separate function so returned errors can be handled and printed
fn try_main() -> Result<(), String> {
    /// Logger setup

    // Get the local time offset for formatting since of course it's my job too
    let local_offset = UtcOffset::current_local_offset()
        .unwrap_or(UtcOffset::UTC);

    // Stdout will have a different log level in debug mode than in release mode
    // Set error level for crates, debug/warn for this app
    let stdout_filter = EnvFilter::try_from_default_env() // allow user to set the filter
        .unwrap_or_else(|_| if cfg!(debug_assertions) { EnvFilter::new("error,WaifuFetcher=debug") }
                            else { EnvFilter::new("error,WaifuFetcher=warn") });
    let stdout_layer = fmt::layer().compact()
        .with_target(true)
        .with_timer(OffsetTime::new(local_offset, timefmt!("[hour]:[minute]:[second]")))
        .with_filter(stdout_filter);

    // Set the path for the log file
    let log_path = data_local_dir()
        .or_else(home_dir)
        .map(|path| path.join(APP_NAME));
    // Create the file layer and organize log files if the path was found (the layer can be None, tracing ignores it)
    // Also keep _guard alive, so logs will be flushed when the app exits
    let (file_layer, _guard) = if let Some(log_path) = log_path {
        if let Err(err) = fs::rename(log_path.join("latest.log"), log_path.join("previous.log")) {
            error!("Error while copying log file: {}", err.to_string());
        }

        let file_appender = tracing_appender::rolling::never(log_path, "latest.log");
        let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);
        let layer = fmt::layer().with_writer(file_writer).with_ansi(false)
            .with_timer(OffsetTime::new(local_offset, timefmt!("[year]-[month]-[day] [hour]:[minute]:[second]")))
            .with_filter(EnvFilter::new("error,WaifuFetcher=info"));

        ( Some(layer), Some(_guard) )
    }
    else { (None, None) };


    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();
    //------------------------------------------------

    info!("Log file for {}\n", APP_NAME);

    // reverse DNS lookup to speed up the first request
    // tokio::net::lookup_host("https://nekos.moe");

    let client = Client::builder()
        .local_addresses(Ipv4Addr::UNSPECIFIED, None) // disable ipv6 (i guess)
        .user_agent(APP_USER_AGENT)
        .build()
        .err_to_string()?;

    let fetcher = Fetcher::new(client);

    // start the iced application
    iced::application(
        move || App::new(fetcher.clone()),
        App::update,
        App::view
    )
    .title("Waifu Fetcher")
    .theme(Theme::KanagawaDragon)
    .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
    .subscription(App::subscription)
    .run()
    .err_to_string()
}