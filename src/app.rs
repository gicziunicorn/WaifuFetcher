#![allow(unused)]
use anyhow::{Context, Result, anyhow};
use bytes::Bytes;
use iced::{
    Element, Subscription, Task, Vector,
    widget::image::Handle,
    keyboard::{
        Key::{self, Named as NamedKey, Character as CharacterKey},
        key::{Named as NamedKeys},
        Event::KeyReleased
    }
};
use image::{ImageFormat, guess_format};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{debug, info, warn};
use waifu_api::fetcher::Fetcher;
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant}
};
use dirs::{home_dir, picture_dir};
use rfd::AsyncFileDialog;
use waifu_api::sites::{Site, Sites};

use crate::update::{self, Message};
use crate::config::Config;
use crate::components::dropdown::{DropdownAction};
use crate::{error, ErrorDetails};
use crate::gui::view;



/// An in-app notification
#[derive(Debug)]
pub struct Notification {
    pub message: String,
    pub ntype: NotificationType,
}

/// Possible notification types
#[derive(Debug)]
pub enum NotificationType {
    Success,
    Error,
}

pub fn notify(msg: impl ToString) -> Message {
    return Message::Notify(msg.to_string());
}

/// Information about the fetched image
#[derive(Debug, Clone)]
pub struct FetchedImage {
    pub id: String,
    pub bytes: Bytes,
    pub handle: Handle,
    pub format: ImageFormat,
}


// Holds values for the states of different elements inside the app.
// These aren't directly changeable from the settings.
#[derive(Debug, Clone)]
pub struct States {
    pub disable_fetch: bool,
    pub dropdown_open: bool,
    pub settings_menu_focused: bool,
    pub zoom: f32,
    pub image_offset: Vector,
    pub auto_fetch_int_input: String,
    pub disabled: bool,
    /// The time left until the next af, in ms
    pub next_auto_fetch: u64,
}
impl Default for States {
    fn default() -> Self {
        let states = States {
            disable_fetch: false,
            dropdown_open: false,
            settings_menu_focused: false,
            zoom: 1.0,
            image_offset: Vector::default(),
            auto_fetch_int_input: String::new(),
            disabled: false,
            next_auto_fetch: 5000,
        };
        states
    }
}


// Holds values that can be changed by the user in the settings.
// This struct is the configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub current_site: Option<Sites>, /// Only store Sites, not Site since there's no need to save the full struct into the config
    // image viewer
    pub fill_screen: bool,
    pub image_save_path: PathBuf,
    pub image_scale_step: f32,
    pub image_min_scale: f32,
    pub image_max_scale: f32,
    pub image_opacity: f32,
    // fetching
    pub fetch_on_startup: bool,
    pub fetch_on_site_change: bool,
    pub auto_fetch: bool,
    /// In ms
    pub auto_fetch_interval: u64,

    pub notification_len: u64,
}
impl Default for Settings {
    fn default() -> Self {
        let save_path = picture_dir()
            .or_else(home_dir)
            .unwrap_or_default();

        Settings {
            current_site: None,

            fill_screen: true,
            image_save_path: save_path,
            image_scale_step: 0.1,
            image_min_scale: 0.25,
            image_max_scale: 10.0,
            image_opacity: 1.0,

            fetch_on_startup: false,
            fetch_on_site_change: false,
            auto_fetch: false,
            auto_fetch_interval: 5000,

            notification_len: 3,
        }
    }
}


#[derive(Debug)]
pub struct AppData {
    pub current_site: Option<Site>,
    pub notifications: Vec<Notification>,
    pub sample_url: &'static str,
    pub file_url: &'static str,
    pub image: Option<FetchedImage>,
    pub zoom: f32,
    pub tags: &'static str,
    pub settings: Settings,
    pub states: States,
    pub in_settings: bool,
}
impl Default for AppData {
    fn default() -> Self {
        AppData {
            current_site: None,
            notifications: Vec::new(),
            sample_url: "",
            file_url: "",
            image: None,
            zoom: 1.0,
            tags: "",
            settings: Settings::default(),
            states: States::default(),
            in_settings: false,
        }
    }
}


#[derive(Debug)]
pub struct App {
    pub fetcher: Fetcher,
    pub appdata: AppData,
    pub config: Option<Config>,
    pub error: Option<String>,
}


impl App {
    /// Initialize the iced app
    pub fn new(fetcher: Fetcher) -> (Self, Task<Message>) {
        debug!("Entered App::new");

        // Initialize the config handler
        let config = match Config::new() {
            Ok(val) => Some(val),
            Err(err) => {
                error!(err.context("Error while loading config"));
                info!("Using default config. Saving the settings to file will also be disabled.");
                None
            },
        };

        // Try to read the config, or fall back to the default
        let settings = if let Some(config) = &config {
            match config.read_config() {
                Ok(settings) => {
                    debug!("Parsed config:\n{:#?}", settings);
                    settings
                }
                Err(err) => {
                    error!( err.context("Failed to load configuration file") );
                    warn!("Using default configuration.");

                    let settings = Settings::default();

                    // Back up the previous config
                    let bak_file = config.file_path.with_added_extension("bak");

                    let copy_result = fs::copy(&config.file_path, &bak_file);
                    if let Err(err) = copy_result {
                        let err = anyhow!(err).context("Failed to create backup");
                        error!(err);
                        warn!("The previous configuration will not be overwritten. Save the settings if you want to overwrite it.");
                    }
                    else {
                        info!("Backed up the previous configuration as '{}'", bak_file.to_string_lossy().to_string());
                        // Save the default config
                        if let Err(err) = config.save_config(&settings) {
                            error!( err.context("Failed to overwrite previous configuration") );
                        }
                    }

                    settings
                }
            }
        } else {
            Settings::default()
        };

        let starting_site = settings.current_site
            .map(|s| s.site(fetcher.clone()));

        // Decide the startup task (whether to fetch or not)
        let startup_task = match settings.fetch_on_startup {
            true => Task::done(Message::FetchImage),
            false => Task::none()
        };

        let mut appdata = AppData {
            settings,
            current_site: starting_site,
            ..Default::default()
        };

        // Set the inputs default values
        appdata.states.auto_fetch_int_input = (appdata.settings.auto_fetch_interval / 1000).to_string();

        // return the task together with Self, this way it will run as the app starts.
        (App {
            fetcher,
            appdata: appdata,
            config,
            error: None,
        },
        startup_task)
    }

    /// The update loop logic of the app
    pub fn update(app: &mut App, message: Message) -> Task<Message> {
        update::update(app, message)
    }

    pub fn view(&self) -> Element<'_, Message> {
        view(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // A subscription that listens to keypresses
        let key_listener = iced::keyboard::listen()
                .filter_map(|keyboard_event| {
                    match keyboard_event {
                        KeyReleased { key, modified_key:_, physical_key:_, location:_, modifiers:_ } => {
                            Some(Message::KeyPressed(key))
                        },
                        _ => None
                    }
                });

        let countdown = if self.appdata.settings.auto_fetch
            && let Some(current_site) = self.appdata.settings.current_site
            && current_site != Sites::NekosBest {
                println!("sub");
                iced::time::every(Duration::from_secs(1))
                    .with(current_site) // A random value that resets the subscription when the site changes
                    .map(|_| Message::Tick)
            } else {
                Subscription::none()
            };

        // Run multiple subscriptions at the same time
        Subscription::batch([key_listener, countdown])
    }

    // pub fn theme(&self) -> Theme { Theme::TokyoNight }
}