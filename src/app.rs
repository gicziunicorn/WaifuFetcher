#![allow(unused)]
use anyhow::{Context, Error, Result, anyhow, bail};
use iced::Vector;
use iced::{Element, Subscription, Task};
use iced_fonts::bootstrap::advanced_text::file;
use image::DynamicImage;
use iced::widget::image::Handle;
use iced::keyboard::{
    Key::{self, Named as NamedKey, Character as CharacterKey},
    key::{Named as NamedKeys},
    Event::KeyReleased
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use waifu_api::fetcher::Fetcher;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
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
    Error(String),
    PickFolder,
    FileSelected(Result<PathBuf, String>),
    /// Fetch an image
    FetchImage,
    /// Handle things after the image was fetched
    FetchedImage( Result<(DynamicImage, Handle), String>, Site ),
    // Handle actions from the site chooser dropdown
    DropdownChanged(DropdownAction<Sites>),
    /// Toggle the settings menu
    ToggleSettings,

    SaveImage,
    SaveCompleted(Result<String, String>),
    KeyPressed(Key),
    SettingsMenuFocused(bool),
    SetViewerScale(f32),
    ChangeViewerScale(i8),
    SetViewerOffset(Vector),
    ChangeImageOpacity(f32),
    SaveSettings,
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
    pub save_path: PathBuf,
    pub image_scale_step: f32,
    pub image_min_scale: f32,
    pub image_max_scale: f32,
    pub image_opacity: f32,
    pub auto_fetch: bool,
}
impl Default for Settings {
    fn default() -> Self {
        let save_path = picture_dir()
            .or_else(home_dir)
            .unwrap_or_default();

        Settings {
            fill_screen: true,
            save_path,
            image_scale_step: 0.1,
            image_min_scale: 0.25,
            image_max_scale: 10.0,
            image_opacity: 1.0,
            auto_fetch: false,
        }
    }
}


#[derive(Debug)]
pub struct AppData {
    pub sites: Sites,
    pub current_site: Option<Site>,
    pub sample_url: &'static str,
    pub file_url: &'static str,
    pub image: Option<DynamicImage>,
    pub image_handle: Option<Handle>,
    pub zoom: f32,
    pub tags: &'static str,
    pub settings: Settings,
    pub states: States,
    pub in_settings: bool,
}
impl Default for AppData {
    fn default() -> Self {
        AppData {
            sites: Sites::NekosMoe,
            current_site: None,
            sample_url: "",
            file_url: "",
            image: None,
            image_handle: None,
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
    pub appdata: Option<AppData>,
    pub config: Option<Config>,
    pub error: Option<String>,
}


impl App {
    pub fn new(fetcher: Fetcher) -> (Self, Task<Message>) {
        debug!("Entered App::new");

        let fetch_on_startup = false;
        let task = if fetch_on_startup { Task::done(Message::FetchImage) }
                    else { Task::none() };

        // Initialize the config
        let config = match Config::new() {
            Ok(val) => Some(val),
            Err(err) => {
                tracing::error!("Error while loading config: \"{}\"", err);
                tracing::error!("Using default config. Saving the settings to file will also be disabled.");
                None
            },
        };

        // Here the config is being parsed, fixed and saved; whatever is needed
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
        }
        else { Settings::default() };

        let appdata = AppData {
            settings,
            ..Default::default()
        };

        // return the task together with Self, this way it will run as soon as possible.
        (App {
            fetcher,
            appdata: Some(appdata),
            config,
            error: None,
        },
        task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Handles errors
            Message::Error(err) => {
                error!(&err);
                self.error = Some(err);
                Task::none()
            },
            // Pick a folder
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
                        Message::FileSelected(result)
                    }
                )
            }
            Message::FileSelected(Ok(path)) => {
                println!("Path selected: {}", path.to_string_lossy().into_owned());
                if let Some(appdata) = &mut self.appdata {
                    appdata.settings.save_path = path;
                }
                Task::none()
            }
            Message::FileSelected(Err(err)) => {
                error!(err);
                Task::none()
            }
            // Handle actions from the site chooser dropdown
            Message::DropdownChanged(action) => {
                if let Some(appdata) = &mut self.appdata {
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
                }
                Task::none()
            }
            // Fetch an image
            Message::FetchImage => {
                if let Some(appdata) = &self.appdata &&
                let Some(current_site) = &appdata.current_site {
                    info!("Fetching image...");
                    // Measure the time of fetching
                    let mut timer = Instant::now();

                    let mut site = current_site.clone();

                    Task::perform(
                        async move {
                            let result = site.get_random_image().await;

                            debug!("Got image in: {:.2}s", timer.elapsed().as_secs_f32());

                            (result, site) // return the site, since its fields could've been modified (like the ratelimits)
                        },
                        |(bytes, site)| {
                            // Measure the time of parsing the image
                            let timer = Instant::now();

                            // Create an image handle and the dynamicimage from the bytes or get the error
                            let result = bytes.context("Image fetching failed!")
                                .and_then(|b| {
                                    let handle = Handle::from_bytes(b.clone());
                                    let image = image::load_from_memory(&b)
                                        .context("Failed to load image")?;

                                    Ok((image, handle))
                                });

                            debug!("Parsed image in: {:.2}s", timer.elapsed().as_secs_f32());

                            // If any, log the full error, then only forward it's message (bc anyhow::Error isn't Clone)
                            let final_result = result.map_err(|err| {
                                error!(&err);
                                format!("{err}")
                            });

                            // Have to forward the site too, because self can't be modified in this closure
                            Message::FetchedImage(final_result, site)
                        }
                    )
                }
                else { Task::none() }
            }
            Message::FetchedImage(result, site) => {
                if let Some(appdata) = &mut self.appdata {
                    // The site could've been modified
                    appdata.current_site = Some(site);

                    match result {
                        Ok((image, handle)) => {
                            appdata.image = Some(image);
                            appdata.image_handle = Some(handle);
                            Task::none()
                        }
                        Err(err) => {
                            self.error = Some(err);
                            Task::none()
                        }
                    }
                }
                else { Task::none() }
            }
            // Toggle the settings menu
            Message::ToggleSettings => {
                if let Some(appdata) = &mut self.appdata {
                    appdata.in_settings = !appdata.in_settings;
                    appdata.states.dropdown_open = false;
                }
                Task::none()
            }

            Message::SaveImage => {
                if let Some(appdata) = self.appdata.as_ref()
                && let Some(image) = appdata.image.as_ref() {
                    let path = appdata.settings.save_path.join("image.jpg");
                    let image_copy = image.clone();
                    return Task::perform(
                        async move {
                            // Enclose blocking CPU/IO operations inside tokio's blocking pool
                            tokio::task::spawn_blocking(move || {
                                image_copy.save(&path)
                                    .map(|_| path.to_string_lossy().into_owned()) // return the path
                                    .map_err(|e| e.to_string())
                            })
                            .await
                            .map_err(|e| e.to_string())?
                        },
                        Message::SaveCompleted,
                    )
                }
                Task::none()
            }
            Message::SaveCompleted(path) => {
                match path {
                    Err(err) => Task::done(Message::Error(err)),
                    Ok(path) => {
                        info!("Saved image as {:?}", path);
                        Task::none()
                    }
                }
            }
            // handle key presses
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
                if let Some(appdata) = &mut self.appdata {
                    appdata.states.settings_menu_focused = is_focused;
                }
                Task::none()
            }
            Message::SetViewerScale(scale) => {
                if let Some(appdata) = &mut self.appdata {
                    appdata.states.zoom = scale;
                }
                Task::none()
            }
            Message::ChangeViewerScale(r) => {
                if let Some(appdata) = &mut self.appdata {
                    if r > 0 && appdata.states.zoom < appdata.settings.image_max_scale {
                        appdata.states.zoom *= 1.0 + appdata.settings.image_scale_step
                    } else
                    if r < 0 && appdata.states.zoom > appdata.settings.image_min_scale {
                        appdata.states.zoom /= 1.0 + appdata.settings.image_scale_step;
                    }
                }

                Task::none()
            }
            Message::SetViewerOffset(offset) => {
                if let Some(appdata) = &mut self.appdata {
                    appdata.states.image_offset = offset;
                }
                Task::none()
            }
            Message::ChangeImageOpacity(change) => {
                if let Some(appdata) = &mut self.appdata {
                    appdata.settings.image_opacity = (appdata.settings.image_opacity + change).clamp(0.0, 1.0);
                }
                Task::none()
            }
            Message::SaveSettings => {
                if let Some(config) = &self.config
                && let Some(appdata) = &self.appdata {
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