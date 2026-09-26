#![allow(unused)]
use iced::Vector;
use iced::{Element, Subscription, Task};
use image::DynamicImage;
use iced::widget::image::Handle;
use iced::keyboard::{
    Key::{self, Named as NamedKey, Character as CharacterKey},
    key::{Named as NamedKeys},
    Event::KeyReleased
};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use dirs::{home_dir, picture_dir};
use rfd::AsyncFileDialog;
use crate::ResultToString;
use crate::config::Config;
use crate::fetcher::{FetchRequest,  Fetcher};
use crate::components::dropdown::{DropdownAction, DropdownItem};
use crate::gui::view;
use crate::sites::Sites;


// Messages that the application uses to notify itself and handle tasks.
#[derive(Debug, Clone)]
pub enum Message {
    Test(String),
    OpenFileDialog,
    FileSelected(Result<PathBuf, String>),
    DownloadedImage( Result<(DynamicImage, Handle, usize, usize), String> ),
    Error( String ),
    DropdownChanged(DropdownAction<Sites>),
    FetchImage,
    PauseTimer,
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
        }
    }
}


pub struct AppData {
    pub sites: Vec<DropdownItem<Sites>>,
    pub choice: Option<Sites>,
    pub sample_url: &'static str,
    pub file_url: &'static str,
    pub image: Option<DynamicImage>,
    pub image_handle: Option<Handle>,
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
    pub tags: &'static str,
    pub settings: Settings,
    pub states: States,
    pub in_settings: bool,
}
impl Default for AppData {
    fn default() -> Self {
        let sites = vec![
            DropdownItem {
                image_path: Sites::Rule34.image_path(),
                value: Sites::Rule34,
            },
            DropdownItem {
                image_path: Sites::Nekos.image_path(),
                value: Sites::Nekos,
            },
        ];
        AppData {
            sites: sites,
            choice: None,
            sample_url: "",
            file_url: "",
            image: None,
            image_handle: None,
            width: 0.0,
            height: 0.0,
            zoom: 1.0,
            tags: "",
            settings: Settings::default(),
            states: States::default(),
            in_settings: false,
        }
    }
}


pub struct App {
    pub fetcher: Fetcher,
    pub appdata: Option<AppData>,
    pub config: Option<Config>,
    pub error: Option<String>,
    timer: Instant,
}


impl App {
    pub fn new(fetcher: Fetcher) -> (Self, Task<Message>) {
        debug!("Entered App::new");

        let fetch_on_startup = false;
        let task = if fetch_on_startup { Task::done(Message::FetchImage) }
                    else { Task::none() };

        // Initialize the config
        let config: Option<Config> = match Config::new() {
            Ok(val) => Some(val),
            Err(err) => {
                error!("Error while loading config: \"{}\"", err);
                error!("Using default config. Saving the settings to file will also be disabled.");
                None
            },
        };

        // Here the config is being parsed, fixed and saved; whatever is needed
        let settings = if let Some(config) = &config {
            match config.read_config() {
                Ok(settings) => {
                    debug!("Read config: {:#?}", settings);
                    settings
                }
                Err(err) => {
                    error!("Error while parsing configuration file: \n'{}'", err);
                    error!("Using default configuration.");

                    info!("Creating backup from current configuration and saving the default settings.");
                    let bak_file = config.file.with_added_extension("bak");

                    let settings = Settings::default();

                    let copy_result = fs::copy(&config.file, &bak_file);
                    if copy_result.is_ok() {
                        let file_name = bak_file.file_name().map(|v| v.to_string_lossy().into_owned()).unwrap_or("{Failed to get file name}".to_string());
                        info!("Backed up the previous configuration as '{}'.", file_name);
                        // Save the default config
                        match config.save_config(&settings) {
                            Err(err) => { error!("Failed to overwrite previous configuration: {}", err); }
                            _ => { info!("Successfully overwrote previous configuration.") }
                        }
                    }
                    else {
                        error!("Failed to create backup: {}", copy_result.err().unwrap().to_string());
                        error!("The configuration will not be overwritten. Save the settings if you want to reset it.");
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
            timer: Instant::now(),
        },
        task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Test(msg) => {
                info!("Received message: {}", msg);
                Task::none()
            }
            Message::OpenFileDialog => {
                Task::perform(
                    async {
                        AsyncFileDialog::new()
                            .pick_folder()
                            .await
                    },
                    |file_handle| {
                        let result = file_handle
                            .ok_or_else(|| String::from("Failed to get file handle for the path."))
                            .and_then(|handle| {
                                Ok(handle.path().to_owned())
                            });
                        Message::FileSelected(result)
                    }
                )
            }
            Message::FileSelected(value) => {
                match value {
                    Ok(path) => {
                        println!("Path selected: {}", path.to_string_lossy().into_owned());
                        if let Some(appdata) = &mut self.appdata {
                            appdata.settings.save_path = path;
                        }
                        Task::none()
                    },
                    Err(e) => Task::done(Message::Error(e)),
                }
            }
            Message::FetchImage => {
                info!("Fetching image...");
                self.timer = Instant::now();

                let appdata = self.appdata.as_ref().unwrap();
                let data = FetchRequest {
                    url: Sites::Nekos.url(), //appdata.choice.unwrap().url(),
                    tags: appdata.tags,
                    limit: "1",
                    api_key: "7d09b9790e4bcf4eecc22c63052cdb3f9f4c00beab93b2932eb0beb32b70d907b23c63210ae7cd33314c55376643ca5f88794777a0828ce77b0b0459836bc5e3",
                    user_id: "4600302",
                };
                let fetcher_clone = self.fetcher.clone();
                Task::perform(
                    async move { fetcher_clone.fetch_site(data, Sites::Nekos).await },
                    |result| {
                        match result {
                            Ok((image_vec, width, height)) => {
                                let handle = Handle::from_bytes(image_vec.clone());
                                let image = image::load_from_memory(&image_vec);
                                match image {
                                    Ok(img) => Message::DownloadedImage(Ok((img, handle, width, height))),
                                    Err(err) => Message::Error(err.to_string())
                                }
                            }
                            Err(err) => {
                                Message::Error(err)
                            }
                        }
                    }
                )
            }
            Message::DownloadedImage(result) => {
                match result {
                    Ok((image, handle, width, height)) => {
                        if let Some(appdata) = self.appdata.as_mut() {
                            appdata.image = Some(image);
                            appdata.image_handle = Some(handle);
                            appdata.width = width as f32;
                            appdata.height = height as f32;
                        }
                        info!("Set image handle");
                        Task::done(Message::PauseTimer)
                    }
                    Err(err) => {
                        Task::done(Message::Error(err))
                    }
                }
            }
            Message::Error(err) => {
                error!("Error: {}", err);
                self.error = Some(err);
                self.appdata = None;
                Task::none()
            }
            Message::DropdownChanged(action) => {
                info!("Site changed");
                let appdata = self.appdata.as_mut().unwrap();
                match action {
                    DropdownAction::Toggle(open_state) => {
                        appdata.states.dropdown_open = open_state;
                    }
                    DropdownAction::Select(chosen_value) => {
                        appdata.choice = Some(chosen_value.clone());
                        appdata.states.dropdown_open = false;
                        info!("user choice: {:?}", chosen_value);
                    }
                }
                Task::none()
            }
            // timer
            Message::PauseTimer => {
                let elapsed = self.timer.elapsed().as_secs_f32();

                info!("Time taken for fetching: {:.2}s", elapsed);

                Task::none()
            }
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
                                    .err_to_string()
                            })
                            .await
                            .err_to_string()?
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
                        Err(err) => { error!("Failed to save current configuration: {}", err); }
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