#![allow(unused)]
#![warn(clippy::unwrap_used)]
#![warn(unused_extern_crates)]
mod app;
mod config;
mod logger;
pub mod gui;
pub mod components;

use anyhow::{Result, Error};
use app::App;
use std::net::Ipv4Addr;
use iced::Theme;
use waifu_api::fetcher::Fetcher;
use wreq::Client;
use tracing::info;

use crate::logger::setup_logger;


static APP_NAME: &str = "WaifuFetcher";

// Create a user agent that can be sent to the API.
// Most sites require or at least recommend this.
static APP_USER_AGENT: &str = "WaifuFetcher (https://github.com/gicziunicorn/WaifuFetcher)";


// Make converting errors to String easier
// trait ResultToString<T> {
//     fn err_to_string(self) -> Result<T, String>;
// }
// impl<T, E: std::fmt::Display> ResultToString<T> for Result<T, E> {
//     fn err_to_string(self) -> Result<T, String> {
//         self.map_err(|e| e.to_string())
//     }
// }

/// Pass errors to the logging function easily
#[macro_export]
macro_rules! error {
    ($err:expr) => {{
        let err = $err;
        let source = err.source().map(|e| e.to_string()).unwrap_or_default();
        tracing::error!(%err, cause = source);
    }};
}



fn main() -> Result<()> {
    let _guard = setup_logger()?;
    info!("Logger set up successfully");

    let client = Client::builder()
        .local_addresses(Ipv4Addr::UNSPECIFIED, None) // disable ipv6 (i guess)
        .user_agent(APP_USER_AGENT)
        .build()?;

    let fetcher = Fetcher::new(client);

    // start the iced application
    iced::application(
        move || App::new(fetcher.clone()),
        App::update,
        App::view
    )
    .title(APP_NAME)
    .theme(Theme::KanagawaDragon)
    .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
    .subscription(App::subscription)
    .run()?;

    Ok(())
}