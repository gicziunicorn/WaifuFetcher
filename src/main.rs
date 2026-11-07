#![allow(unused)]
#![warn(clippy::unwrap_used)]
#![warn(unused_extern_crates)]
mod app;
mod config;
mod logger;
pub mod gui;
pub mod components;
mod update;

use anyhow::{Context, Error, Result};
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

//-- Make errors easily loggable --
/// Shared methods for error values
trait ErrorDetails {
    fn error_msg(&self) -> String;
    fn causes(&self) -> Vec<String>;
}

// implement the trait for errors
impl ErrorDetails for anyhow::Error {
    fn error_msg(&self) -> String {
        self.to_string()
    }

    fn causes(&self) -> Vec<String> {
        // Get the chain, skip the main error, and collect the causes (contexts) into a vec
        self.chain()
            .skip(1)
            .map(|e| e.to_string())
            .collect()
    }
}

impl ErrorDetails for str {
    fn error_msg(&self) -> String {
        self.to_string()
    }

    fn causes(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Helper macro to pass arguments to tracing::error
#[macro_export]
macro_rules! error {
    ($err:expr) => {{
        let err = $err;
        let error_msg = err.error_msg();
        let causes = err.causes();

        tracing::error!(
            error = %error_msg,
            causes = %causes.join("|$|"),
        );
    }};
}
//----


fn main() -> Result<()> {
    let _guard = setup_logger()
        .context("Failed to set up logger.")?;
    info!("Logger set up successfully");

    let client = Client::builder()
        .local_addresses(Ipv4Addr::UNSPECIFIED, None) // disable ipv6 (i guess)
        .user_agent(APP_USER_AGENT)
        .build()
        .context("Building the client failed")?;

    let fetcher = Fetcher::new(client);

    // start the iced application
    iced::application(
        move || App::new(fetcher.clone()),
        App::update,
        App::view
    )
    .title(APP_NAME)
    .theme(Theme::KanagawaDragon)
    // Add the bootstrap font for icons
    .font(iced_fonts::BOOTSTRAP_FONT_BYTES)
    .subscription(App::subscription)
    .run()
    .context("Failed to run the iced application")?;

    Ok(())
}