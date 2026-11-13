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
use waifu_api::{error::FetchResult, fetcher::Fetcher, sites::Post};
use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant}
};
use dirs::{home_dir, picture_dir};
use rfd::AsyncFileDialog;
use waifu_api::sites::{Site, Sites};

use crate::{
    ErrorDetails, app::{App, FetchedImage, Notification, NotificationType, notify}, components::dropdown::DropdownAction, config::Config, error, update::Message::Notify,
};


// Messages that the application uses to notify itself and handle tasks.
#[derive(Debug, Clone)]
pub enum Message {
    Disable(bool),
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
    /// Toggle the settings menu
    ToggleSettings,
    /// Save the current image
    SaveImage,

    /// Smaller interactions with interactable elements (inputs, togglers, etc...)
    Interaction(Interaction),

    KeyPressed(Key),
    SettingsMenuFocused(bool),
    SetViewerScale(f32),
    ChangeViewerScale(i8),
    SetViewerOffset(Vector),
    ChangeImageOpacity(f32),
    SaveSettings,
}

#[derive(Debug, Clone)]
pub enum Interaction {
    SiteDropdown(DropdownAction<Sites>),
    ToggleAutoFetch(bool),
    AutoFetchIntervalInput(String),
    AutoFetchIntervalSubmit,
}


pub fn update(app: &mut App, message: Message) -> Task<Message> {
    let appdata = &mut app.appdata;

    match message {
        Message::Disable(val) => {appdata.states.disabled = val; Task::none()},
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
            let path_s = path.to_string_lossy().into_owned();

            appdata.settings.image_save_path = path;

            debug!("Path selected: {}", path_s);
            Task::done( Notify(format!("Set path to: {path_s}")) )
        }
        Message::PathSelected(Err(err)) => {
            error!(err);
            Task::none()
        }
        // Fetch an image
        Message::FetchImage => {
            if let Some(current_site) = &appdata.current_site {
                appdata.states.disable_fetch = true;

                // Measure the time of fetching
                let timer = Instant::now();

                let mut site = current_site.clone();
                let fetcher = app.fetcher.clone();

                Task::perform(
                    async move {
                        // Fetch a post
                        let res: FetchResult<Post> = site.get_random_posts(1).await
                                                        .map(|mut posts| posts.remove(0))
                                                        .map(Into::into);

                        // If the post was fetched, fetch an image, else return the error
                        let result: FetchResult<(Post, Bytes)> = match res {
                            Ok(post) => {
                                fetcher.fetch_image(&post.image_url).await
                                    .map(|bytes| (post, bytes))
                            },
                            Err(e) => Err(e),
                        };

                        debug!("Got response in: {:.2}s", timer.elapsed().as_secs_f32());

                        (result, site) // return the site, since its fields could've been modified (like the ratelimits)
                    },
                    |(result, site)| {
                        // Create an image handle and the dynamicimage from the bytes or get the error
                        let result = result.context("Failed to fetch image")
                            .and_then(|(post, bytes)| {
                                let handle = Handle::from_bytes(bytes.clone());

                                let format = guess_format(&bytes)
                                    .context("Failed to determine image format")?;

                                let image = FetchedImage {
                                    id: post.id,
                                    bytes,
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
            else {
                Task::done( Message::Notify("No site selected".to_string()) )
            }
        }
        Message::FetchedImage(result, site) => {
            // The site could've been modified
            appdata.current_site = Some(site);
            appdata.states.disable_fetch = false;

            match result {
                Ok(fetched_image) => {
                    appdata.image = Some(fetched_image);
                    Task::none()
                }
                Err(err) => {
                    Task::done(Message::Notify(err))
                }
            }

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
                let name = &image.id;
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
                            Ok(path) => {
                                let msg = format!("Saved image as {path}");
                                Message::Notify(msg)
                            },
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
                        NamedKeys::F1 => Task::done(Message::ToggleSettings),
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
            if let Some(config) = &app.config {
                match config.save_config(&appdata.settings) {
                    Err(err) => { tracing::error!("Failed to save current configuration: {}", err); }
                    _ => { info!("Saved current configuration.") }
                }
            }
            Task::none()
        }

        Message::Interaction(interaction) => {
            match interaction {
                // Handle actions from the site chooser dropdown
                Interaction::SiteDropdown(action) => {
                    match action {
                        DropdownAction::Toggle(open_state) => {
                            appdata.states.dropdown_open = open_state;
                        }
                        DropdownAction::Select(chosen_value) => {
                            appdata.settings.current_site = Some(chosen_value);
                            let site = chosen_value.site(app.fetcher.clone());
                            appdata.current_site = Some(site);
                            appdata.states.dropdown_open = false;
                            debug!("Set site to: {:?}", chosen_value);
                        }
                    }
                    Task::none()
                }
                Interaction::ToggleAutoFetch(val) => {
                    appdata.settings.auto_fetch = val;
                    Task::none()
                }
                Interaction::AutoFetchIntervalInput(input) => {
                    if input.len() > if input.contains('.') && !input.ends_with('.') { 5 } else { 4 } {
                        return Task::none();
                    }

                    if input.ends_with('.') || input == "" {
                        appdata.states.auto_fetch_int_input = input;
                        return Task::none();
                    }

                    if input.parse::<f64>().is_err() {
                        return Task::none();
                    };

                    appdata.states.auto_fetch_int_input = input;

                    Task::none()
                }
                Interaction::AutoFetchIntervalSubmit => {
                    let input = &appdata.states.auto_fetch_int_input;

                    if input == "" {
                        return Task::none();
                    }

                    let Ok(parsed_input) = input.parse::<f64>() else {
                        return Task::done(notify("Failed to parse input!"));
                    };

                    if !parsed_input.is_finite() {
                        return Task::done(notify("Try a bit smaller value..."));
                    }
                    if parsed_input < 1.0 {
                        return Task::done(notify("Value can't be smaller than one!"));
                    }

                    let interval = (parsed_input * 1000.0).round() as u64;

                    appdata.settings.auto_fetch_interval = interval;

                    Task::none()
                }
            }
        }
    }
}