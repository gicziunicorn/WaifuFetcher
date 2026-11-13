#![warn(unused)]
use iced::Length::Fill;
use iced::{
    Element,
    widget::{column, container, text},
};

use crate::{
    app::AppData,
    components::image_viewer::Viewer,
    gui::{bottom_bar::bottom_bar, navbar::navbar},
    update::Message,
};


pub fn main_screen(appdata: &AppData) -> Element<'_, Message> {
    column!(
        navbar(&appdata.current_site, appdata.states.dropdown_open, appdata.states.disable_fetch),
        picture_displayer(appdata),
        bottom_bar(appdata),
    )
    .into()
}


pub fn picture_displayer(appdata: &AppData) -> Element<'_, Message> {
    if let Some(image) = &appdata.image {
        let image_viewer = Viewer::new(
            &image.handle,
            Fill,
            Fill,
            appdata.states.zoom,
            appdata.states.image_offset,
            appdata.settings.image_opacity,
            Message::SetViewerScale,
            Message::SetViewerOffset,
        );

        container(image_viewer).center_x(Fill).center_y(Fill).into()
    } else if appdata.current_site.is_some() {
        container(text("Fetch an image!").size(24))
            .center_x(Fill)
            .center_y(Fill)
            .into()
    } else {
        container(text("Select a site!").size(24))
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
}
