#![allow(unused)]
use iced::Length::Fill;
use iced::keyboard::Key;
use iced::time;
use iced::widget::pick_list;
use iced::{
    Alignment, Border, Color, Element, Subscription, Theme, Task,
    widget::{
        Slider, Space, button, column, image as iced_image, row, slider, text, text_editor, container,
        container::{Style, background},
    },
    Event::{
        Mouse as MouseEvent, Keyboard as KeyboardEvent,
    },
    mouse::{
        ScrollDelta,
        Event::WheelScrolled as ScrollEvent,
    },
    keyboard::{
        Key::Named as NamedKey,
        key::Named as NamedKeys,
        Event::KeyReleased
    },
};
use iced_image::Handle;
use image::DynamicImage;
use log::info;
use std::fmt::format;
use std::fs::{read, File};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::io::Read;
use std::str::from_utf8;
use rfd::AsyncFileDialog;
use crate::app::Message::PauseTimer;
use crate::fetcher::{FetchRequest, FetchRequestType, Fetcher};
use wreq::Client;
use log::error;
use crate::components::dropdown::{DropdownAction, DropdownItem, image_dropdown};
use crate::gui::view;
use crate::sites::{Rule34Response, Sites};
use std::time::{Duration, Instant, SystemTime};



#[derive(Debug, Clone)]
pub enum Message {
    Test(String),
    OpenFileDialog,
    FileSelected(Result<String, String>),
    DownloadedImage( Result<(DynamicImage, Handle, usize, usize), String> ),
    Error( String ),
    DropdownChanged(DropdownAction<Sites>),
    FetchImage,
    PauseTimer,
    ToggleSettings,
    SaveImage,
    SaveCompleted(Result<String, String>),
    KeyPressed(Key),
}

#[derive(Debug, Clone)]
pub struct MiscData {
    pub dropdown_open: bool,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub fill_screen: bool,
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
    pub misc: MiscData,
    pub save_path: String,
}

pub struct App {
    pub fetcher: Fetcher,
    pub appdata: Option<AppData>,
    pub error: Option<String>,
    pub settings: bool,
    timer: Instant,
}


impl App {
    pub fn new(fetcher: Fetcher) -> (Self, Task<Message>) {
        info!("App::new");

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

        let appdata = AppData {
            sites,
            choice: Some(Sites::Nekos),
            sample_url: "",
            file_url: "",
            image: None,
            image_handle: None,
            width: 0.0,
            height: 0.0,
            zoom: 1.0,
            tags: "",
            settings: Settings {
                fill_screen: true,
            },
            misc: MiscData {
                dropdown_open: false,
            },
            save_path: String::from(""),
        };

        info!("Tags: {}", appdata.tags);


        let start_site = appdata.choice.unwrap();
        // initial fetch when the app starts
        let data = FetchRequest {
            url: start_site.url(),
            tags: appdata.tags,
            limit: "1",
            api_key: "7d09b9790e4bcf4eecc22c63052cdb3f9f4c00beab93b2932eb0beb32b70d907b23c63210ae7cd33314c55376643ca5f88794777a0828ce77b0b0459836bc5e3",
            user_id: "4600302",
        };
        let fetcher_clone = fetcher.clone();
        let fetch_on_startup = false;
        let fetch = if fetch_on_startup { Task::done(Message::FetchImage) }
                    else { Task::none() };

        (App {
            fetcher,
            appdata: Some(appdata),
            error: None,
            settings: false,
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
                                handle
                                    .path()
                                    .to_str()
                                    .map(|p| p.to_string()) // Convert &str to owned String
                                    .ok_or_else(|| String::from("The path is not a valid UTF-8 string."))
                            });
                        Message::FileSelected(result)
                    }
                )
            }
            Message::FileSelected(value) => {
                match value {
                    Ok(path) => {
                        println!("Path selected: {}", path);
                        if let Some(appdata) = &mut self.appdata {
                            appdata.save_path = format!("{}/", path);
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
                    url: appdata.choice.unwrap().url(),
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
                        appdata.misc.dropdown_open = open_state;
                    }
                    DropdownAction::Select(chosen_value) => {
                        appdata.choice = Some(chosen_value.clone());
                        appdata.misc.dropdown_open = false;
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
                self.settings = !self.settings;
                let appdata = self.appdata.as_mut().unwrap();
                appdata.misc.dropdown_open = false;
                Task::none()
            }
            Message::SaveImage => {
                if let Some(appdata) = self.appdata.as_ref()
                && let Some(image) = appdata.image.as_ref() {
                    let path = format!("{}image.jpg", appdata.save_path);
                    let image_copy = image.clone();
                    return Task::perform(
                        async move {
                            // Enclose blocking CPU/IO operations inside tokio's blocking pool
                            tokio::task::spawn_blocking(move || {
                                image_copy.save(PathBuf::from(&path))
                                    .map(|_| path) // return the path
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

    pub fn theme(&self) -> Theme { Theme::TokyoNight }
}