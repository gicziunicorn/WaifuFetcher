#![allow(unused)]
use iced::Vector;
use iced::keyboard::Key;
use iced::{
    Element, Subscription, Theme, Task,
    keyboard::{
        Key::Named as NamedKey,
        key::Named as NamedKeys,
        Event::KeyReleased
    },
};
use iced::widget::image::Handle;
use image::DynamicImage;
use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use rfd::AsyncFileDialog;
use crate::APP_NAME;
use crate::fetcher::{FetchRequest,  Fetcher};
use log::error;
use crate::components::dropdown::{DropdownAction, DropdownItem};
use crate::gui::view;
use crate::sites::{Site, Sites};
use std::time::Instant;
use dirs::{home_dir, picture_dir, config_local_dir};
use serde_json as json;



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
    ImageViewer(f32, Vector)
}


#[derive(Debug, Clone)]
pub struct States {
    pub dropdown_open: bool,
    pub settings_menu_focused: bool,
}
impl Default for States {
    fn default() -> Self {
        States {
            dropdown_open: false,
            settings_menu_focused: false,
        }
    }
}


// Holds values that can be changed by the user in the settings
// This struct will be as the configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub fill_screen: bool,
    pub save_path: PathBuf,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            fill_screen: true,
            save_path: PathBuf::default(),
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
    pub error: Option<String>,
    timer: Instant,
}


impl App {
    pub fn new(fetcher: Fetcher) -> (Self, Task<Message>) {
        info!("Entered App::new");

        // Read the config first
        fn get_config() -> Result<Settings, String> {
            // First try to find the local config directory
            let config_file_path = config_local_dir()
                .or_else(home_dir)
                .ok_or_else(|| "Failed to find config folder.".to_string()) ?
                // Then find the file
                .join(APP_NAME)
                .join("config.toml");

            // Read the file's content
            let bytes = fs::read(config_file_path)
                .map_err(|e| e.to_string())?;

            // Then just deserialize it into the struct
            toml::from_slice(&bytes).map_err(|e| e.to_string())
        }

        let appdata = match get_config() {
            Ok(settings) => {
                info!("Result: {:#?}", settings);
                AppData {
                    settings,
                    ..Default::default()
                }
            }
            Err(err) => {
                error!("Error while loading config file: '{}', using default config. Saving the settings to file will also be disabled.", err);
                AppData::default()
            }
        };


        let json = json::to_string_pretty(&appdata.settings);
        match json {
            Ok(data) => {
                info!("Json data:\n{}", data);
            }
            Err(e) => {
                info!("Couldn't create json data from settings: {}", e.to_string());
            }
        }
        let toml = toml::to_string_pretty(&appdata.settings);
        match toml {
            Ok(data) => {
                info!("Toml data:\n{}", data);
            }
            Err(e) => {
                info!("Couldn't create toml data from settings: {}", e.to_string());
            }
        }

        // info!("Set config directory to: {:?}", app_config_folder);
        // fs::create_dir_all(app_config_folder);


        let save_path = picture_dir()
            .or_else(home_dir)
            //.map(|v| v.to_string_lossy().into_owned())
            .unwrap_or_default();
        info!("Set save directory to: {:?}", save_path);

        /*let appdata = AppData {
            choice: Some(Sites::Nekos),
            settings: Settings {
                save_path,
                ..Default::default()
            },
            ..Default::default()
        };*/

        // info!("Tags: {}", appdata.tags);


        // let start_site = appdata.choice.unwrap();
        // initial fetch when the app starts
        /* let data = FetchRequest {
            url: start_site.url(),
            tags: appdata.tags,
            limit: "1",
            api_key: "7d09b9790e4bcf4eecc22c63052cdb3f9f4c00beab93b2932eb0beb32b70d907b23c63210ae7cd33314c55376643ca5f88794777a0828ce77b0b0459836bc5e3",
            user_id: "4600302",
        };*/
        let fetch_on_startup = false;
        let fetch = if fetch_on_startup { Task::done(Message::FetchImage) }
                    else { Task::none() };

        (App {
            fetcher,
            appdata: Some(appdata),
            error: None,
            timer: Instant::now(),
        },
        fetch)
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
            Message::KeyPressed(key) => {
                // println!("Pressed key: {:?}", key);
                match key {
                    NamedKey(named) => {
                        match named {
                            NamedKeys::Escape => Task::done(Message::ToggleSettings),
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
            Message::ImageViewer(scale, offset) => {
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