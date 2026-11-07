// Contains the main view() function for the app
// and act as a mod.rs for the files inside gui/
#![allow(unused)]
use iced::Length::Fill;
use iced::widget::stack;
use iced::{
    Element,
    widget::{
        text, container,
    }
};
use crate::app::App;
use crate::gui::notification::notifications;
use crate::gui::{main_screen::main_screen, settings_screen::settings_screen};
use crate::update::Message;

mod main_screen;
mod settings_screen;
mod notification;
mod bottom_bar;


/// The main view of the app
pub fn view(appstate: &App) -> Element<'_, Message> {
    let appdata = &appstate.appdata;

    let main_screen = main_screen(appdata);
    let settings_screen = if appdata.in_settings {
        Some(settings_screen(&appdata.settings, &appdata.states))
    } else { None };
    let notifications = notifications(&appdata.notifications);

    stack!(
        main_screen,
        settings_screen,
        notifications,
    ).into()
}

