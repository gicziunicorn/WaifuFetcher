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
use crate::app::{Message, App};
use crate::gui::{main_screen::main_screen, settings_screen::settings_screen};

mod main_screen;
mod settings_screen;


pub fn view(appstate: &App) -> Element<'_, Message> {
    let appdata = &appstate.appdata;
    let main_screen = main_screen(appdata);
    if !appdata.in_settings {
        main_screen
    }
    else {
        stack!(
            main_screen,
            settings_screen(&appdata.settings, appdata.states.settings_menu_focused)
        ).into()
    }

    /*println!("rerendered");
    stack!(
        main_screen(appdata),
        settings_screen(appdata)
    ).into()*/

    // If appdata is not found then show the error message.
    // let error = appstate.error.as_deref().unwrap_or_default();
    // container(
    //     text(format!("An error occurred: {}", error)).size(20)
    // )
    // .center_x(Fill).center_y(Fill)
    // .into()
}

