#![allow(unused)]
use anyhow::{Context, Result};
use bytes::Bytes;
use iced::Vector;
use iced::{Element, Subscription, Task};
use image::{DynamicImage, GenericImageView, ImageFormat, guess_format};
use iced::widget::image::Handle;
use iced::keyboard::{
    Key::{self, Named as NamedKey, Character as CharacterKey},
    key::{Named as NamedKeys},
    Event::KeyReleased
};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{debug, info, warn};
use waifu_api::fetcher::Fetcher;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use dirs::{home_dir, picture_dir};
use rfd::AsyncFileDialog;
use waifu_api::site::{Site, Sites};
use crate::config::Config;
use crate::components::dropdown::{DropdownAction};
use crate::{ErrorDetails, error};
use crate::gui::view;


// Messages that the application uses to notify itself and handle tasks.
#[derive(Debug, Clone)]
pub enum Message {
    /// Do nothing
    None,
    /// Post a notification
    Notify(String),
    /// Delete old notifications
    NotificationEnd,
    /// Let the user pick a folder
    PickFolder,
    /// Set the path where the pictures will be saved
    PathSelected(Result<PathBuf, String>),
    /// Fetch an image
    FetchImage,
    /// Handle things after the image was fetched
    FetchedImage( Result<FetchedImage, String>, Site ),
    // Handle actions from the site chooser dropdown
    DropdownChanged(DropdownAction<Sites>),
    /// Toggle the settings menu
    ToggleSettings,
    /// Save the current image
    SaveImage,

    KeyPressed(Key),
    SettingsMenuFocused(bool),
    SetViewerScale(f32),
    ChangeViewerScale(i8),
    SetViewerOffset(Vector),
    ChangeImageOpacity(f32),
    SaveSettings,
}


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

/// Information about the fetched image
#[derive(Debug, Clone)]
pub struct FetchedImage {
    pub bytes: Bytes,
    pub handle: Handle,
    pub format: ImageFormat,
}


// Holds values for the states of different elements inside the app.
// These aren't directly changeable from the settings.
#[derive(Debug, Clone)]
pub struct States {
    pub dropdown_open: bool,
    pub settings_menu_focused: bool,
    pub zoom: f32,
    pub image_offset: Vector,
}
impl Default for States {
    fn default() -> Self {
        States {
            dropdown_open: false,
            settings_menu_focused: false,
            zoom: 1.0,
            image_offset: Vector::default(),
        }
    }
}


// Holds values that can be changed by the user in the settings.
// This struct is the configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub fill_screen: bool,
    pub image_save_path: PathBuf,
    pub image_scale_step: f32,
    pub image_min_scale: f32,
    pub image_max_scale: f32,
    pub image_opacity: f32,
    pub fetch_on_startup: bool,
    pub auto_fetch: bool,
    pub notification_len: u64,
}
impl Default for Settings {
    fn default() -> Self {
        let save_path = picture_dir()
            .or_else(home_dir)
            .unwrap_or_default();

        Settings {
            fill_screen: true,
            image_save_path: save_path,
            image_scale_step: 0.1,
            image_min_scale: 0.25,
            image_max_scale: 10.0,
            image_opacity: 1.0,
            fetch_on_startup: false,
            auto_fetch: false,
            notification_len: 3,
        }
    }
}


#[derive(Debug)]
pub struct AppData {
    pub notifications: Vec<Notification>,
    pub sites: Sites,
    pub current_site: Option<Site>,
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
            notifications: Vec::new(),
            sites: Sites::NekosMoe,
            current_site: None,
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
                    debug!("Read config:\n{:#?}", settings);
                    settings
                }
                Err(err) => {
                    tracing::error!("Error while parsing configuration file: \n'{}'", err);
                    tracing::error!("Using default configuration.");

                    info!("Creating backup from current configuration and saving the default settings.");
                    let bak_file = config.file_path.with_added_extension("bak");

                    let settings = Settings::default();

                    let copy_result = fs::copy(&config.file_path, &bak_file);
                    if copy_result.is_ok() {
                        let file_name = bak_file.file_name().map(|v| v.to_string_lossy().into_owned()).unwrap_or("{Failed to get file name}".to_string());
                        info!("Backed up the previous configuration as '{}'.", file_name);
                        // Save the default config
                        match config.save_config(&settings) {
                            Err(err) => { tracing::error!("Failed to overwrite previous configuration: {}", err); }
                            _ => { info!("Successfully overwrote previous configuration.") }
                        }
                    }
                    else {
                        tracing::error!("Failed to create backup: {}", copy_result.err().unwrap().to_string());
                        tracing::error!("The configuration will not be overwritten. Save the settings if you want to reset it.");
                    }

                    settings
                }
            }
        } else {
            Settings::default()
        };

        // Decide the startup task (whether to fetch or not)
        let startup_task = match settings.fetch_on_startup {
            true => Task::done(Message::FetchImage),
            false => Task::none()
        };

        let appdata = AppData {
            settings,
            ..Default::default()
        };

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
    pub fn update(&mut self, message: Message) -> Task<Message> {
        let appdata = &mut self.appdata;
        match message {
            // Do nothing
            Message::None => Task::none(),
            /// Post a notification
            Message::Notify(msg) => {
                if appdata.notifications.len() >= 5 {
                    appdata.notifications.remove(0);
                }

                let notif = Notification {
                    message: msg,
                    ntype: NotificationType::Error
                };

                appdata.notifications.push(notif);

                // Delete the notification after n secs
                let len = appdata.settings.notification_len;
                Task::perform(
                    async move { sleep(Duration::from_secs(len)).await; },
                    |_| { Message::NotificationEnd }
                )
            },
            Message::NotificationEnd => {
                // remove the first notification
                if appdata.notifications.len() > 0 {
                    appdata.notifications.remove(0);
                }
                Task::none()
            },
            // Let the user pick a folder
            Message::PickFolder => {
                debug!("Launching file picker");
                Task::perform(
                    async {
                        AsyncFileDialog::new()
                            .pick_folder()
                            .await
                    },
                    |file_handle| {
                        let result = file_handle.ok_or(String::from("No folder path received from the filepicker."))
                            .map(|f| f.path().to_owned());
                        Message::PathSelected(result)
                    }
                )
            }
            // Set the path where the pictures will be saved
            Message::PathSelected(Ok(path)) => {
                println!("Path selected: {}", path.to_string_lossy().into_owned());
                appdata.settings.image_save_path = path;
                Task::none()
            }
            Message::PathSelected(Err(err)) => {
                error!(err);
                Task::none()
            }
            // Handle actions from the site chooser dropdown
            Message::DropdownChanged(action) => {
                match action {
                    DropdownAction::Toggle(open_state) => {
                        appdata.states.dropdown_open = open_state;
                    }
                    DropdownAction::Select(chosen_value) => {
                        let site = chosen_value.site(self.fetcher.clone());
                        appdata.current_site = Some(site);
                        appdata.states.dropdown_open = false;
                        debug!("Set site to: {:?}", chosen_value);
                    }
                }
                Task::none()
            }
            // Fetch an image
            Message::FetchImage => {
                if let Some(current_site) = &appdata.current_site {
                    // Measure the time of fetching
                    let timer = Instant::now();

                    let mut site = current_site.clone();

                    Task::perform(
                        async move {
                            let result = site.get_random_image().await;

                            debug!("Got image in: {:.2}s", timer.elapsed().as_secs_f32());

                            (result, site) // return the site, since its fields could've been modified (like the ratelimits)
                        },
                        |(bytes, site)| {
                            // Create an image handle and the dynamicimage from the bytes or get the error
                            let result = bytes.context("Image fetching failed!")
                                .and_then(|b| {
                                    let handle = Handle::from_bytes(b.clone());

                                    let format = guess_format(&b)
                                        .context("Failed to determine image format")?;

                                    let image = FetchedImage {
                                        bytes: b,
                                        handle,
                                        format
                                    };

                                    Ok(image)
                            });

                            // If any, log the full error, then only forward it's message (bc anyhow::Error isn't Clone)
                            let final_result = result.map_err(|err| {
                                error!(&err);
                                format!("{err}")
                            });

                            // Have to forward the site too, because self (appdata) can't be modified in this closure
                            Message::FetchedImage(final_result, site)
                        }
                    )
                }
                else { Task::none() }
            }
            Message::FetchedImage(result, site) => {
                // The site could've been modified
                appdata.current_site = Some(site);

                match result {
                    Ok(fetched_image) => {
                        appdata.image = Some(fetched_image);
                    }
                    Err(err) => {
                        self.error = Some(err);
                    }
                }

                Task::none()
            }
            // Toggle the settings menu
            Message::ToggleSettings => {
                appdata.in_settings = !appdata.in_settings;
                appdata.states.dropdown_open = false;
                Task::none()
            }
            // Save the current image
            Message::SaveImage => {
                if let Some(image) = &appdata.image {
                    // Get the name and extension of the image from `ImageFormat` and construct the path
                    let name = "image";
                    let ext = image.format.extensions_str()[0];
                    let path = appdata.settings.image_save_path.join(format!("{name}.{ext}"));

                    let image_copy = image.clone();
                    return Task::perform(
                        async move {
                            debug!("Saving image as {:?}", path);
                            // Enclose the operation inside tokio's blocking pool
                            tokio::task::spawn_blocking(move || {
                                fs::write(&path, &image_copy.bytes)
                                    .map(|_| path.to_string_lossy().into_owned()) // return the path
                                    .context("Failed to save image")
                            })
                            .await
                            .context("Failed to save image")
                        },
                        |result| {
                            match result.flatten() {
                                Ok(path) => Message::Notify( format!("Saved image to {path}") ),
                                Err(err) => {
                                    error!(&err);
                                    Message::Notify(err.to_string())
                                }
                            }
                        }
                    )
                }
                Task::none()
            }

            // Handle key presses
            Message::KeyPressed(key) => {
                //debug!("Key pressed: {:?}", key);
                match key.as_ref() {
                    NamedKey(named) => {
                        match named {
                            NamedKeys::Escape => Task::done(Message::ToggleSettings),
                            NamedKeys::ArrowUp => Task::done(Message::ChangeViewerScale(1)),
                            NamedKeys::ArrowDown => Task::done(Message::ChangeViewerScale(-1)),
                            _ => Task::none()
                        }
                    },
                    CharacterKey(char) => {
                        match char {
                            "+" => Task::done(Message::ChangeImageOpacity(0.1)),
                            "-" => Task::done(Message::ChangeImageOpacity(-0.1)),
                            _ => Task::none()
                        }
                    },
                    _ => Task::none()
                }
            }
            Message::SettingsMenuFocused(is_focused) => {
                appdata.states.settings_menu_focused = is_focused;
                Task::none()
            }
            Message::SetViewerScale(scale) => {
                appdata.states.zoom = scale;
                Task::none()
            }
            Message::ChangeViewerScale(r) => {
                if r > 0 && appdata.states.zoom < appdata.settings.image_max_scale {
                    appdata.states.zoom *= 1.0 + appdata.settings.image_scale_step
                } else
                if r < 0 && appdata.states.zoom > appdata.settings.image_min_scale {
                    appdata.states.zoom /= 1.0 + appdata.settings.image_scale_step;
                }

                Task::none()
            }
            Message::SetViewerOffset(offset) => {
                appdata.states.image_offset = offset;
                Task::none()
            }
            Message::ChangeImageOpacity(change) => {
                appdata.settings.image_opacity = (appdata.settings.image_opacity + change).clamp(0.0, 1.0);
                Task::none()
            }
            Message::SaveSettings => {
                if let Some(config) = &self.config {
                    match config.save_config(&appdata.settings) {
                        Err(err) => { tracing::error!("Failed to save current configuration: {}", err); }
                        _ => { info!("Saved current configuration.") }
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        view(self)
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::keyboard::listen()
            .filter_map(|keyboard_event| {
                match keyboard_event {
                    KeyReleased { key, modified_key:_, physical_key:_, location:_, modifiers:_ } => {
                        Some(Message::KeyPressed(key))
                    },
                    _ => None
                }
            })
    }

    // pub fn theme(&self) -> Theme { Theme::TokyoNight }
}