#![allow(unused)]
mod app;
mod fetcher;
mod sites;
pub mod gui;
pub mod components;

use std::net::Ipv4Addr;
use iced::Theme;
use wreq::Client;
use wreq_util::Emulation;
use log::{error, info, debug};
use iced_fonts::BOOTSTRAP_FONT_BYTES;

use app::App;
use fetcher::Fetcher;


static APP_NAME: &str = "WaifuFetcher";

static API_KEY: &str = "7d09b9790e4bcf4eecc22c63052cdb3f9f4c00beab93b2932eb0beb32b70d907b23c63210ae7cd33314c55376643ca5f88794777a0828ce77b0b0459836bc5e3";
static USER_ID: &str = "4600302";

pub fn main() {
    env_logger::Builder::from_default_env().init();

    // reverse DNS lookup to speed up the first request
    // tokio::net::lookup_host("https://nekos.moe");

    if let Err(e) = try_main() {
        error!("An error occurred: {}", e);
        std::process::exit(1);
    }
}

// separate function so returned errors can be handled and printed
fn try_main() -> Result<(), String> {
    let client = Client::builder()
        .local_addresses(Ipv4Addr::UNSPECIFIED, None) // disable ipv6 (i guess)
        .emulation(Emulation::Chrome137)
        .build()
        .map_err(|e| e.to_string())?;

    let fetcher = Fetcher::new(client.clone());

    // start the iced application
    iced::application(
        move || App::new(fetcher.clone()),
        App::update,
        App::view
    )
    .title("Waifu Fetcher")
    .theme(Theme::KanagawaDragon)
    .font(BOOTSTRAP_FONT_BYTES)
    .subscription(App::subscription)
    .run().map_err(|e| e.to_string())
}