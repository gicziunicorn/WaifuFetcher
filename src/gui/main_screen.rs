#![warn(unused)]
use strum::IntoEnumIterator;
use iced::Length::Fill;
use iced::{
    Alignment, Border, Color, Element, Theme,
    widget::{ Space, button, column, row, text, container, container::Style }
};
use waifu_api::site::{Site, Sites};
use crate::APP_NAME;
use crate::components::image_viewer::Viewer;
use crate::components::dropdown::image_dropdown;
use crate::app::AppData;
use crate::update::{Interaction, Message};


pub fn main_screen(appdata: &AppData) -> Element<'_, Message> {
    column!(
        navbar(
            &appdata.current_site,
            appdata.states.dropdown_open,
        ),
        picture_displayer(appdata)
    ).into()
}

pub fn navbar(current_site: &Option<Site>, dropdown_open: bool) -> Element<'_, Message> {
    container(
        row![
            Space::new().width(20),
            button( text("Fetch").color(Color::WHITE) )
                .on_press(Message::FetchImage)
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => { button::Style::default().with_background(palette.background.stronger.color) }
                        _ => button::Style::default().with_background(palette.background.strong.color),
                    }
                }),
            Space::new().width(20),
            button( text("Save image").color(Color::WHITE) )
                .on_press(Message::SaveImage)
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => { button::Style::default().with_background(palette.background.stronger.color) }
                        _ => button::Style::default().with_background(palette.background.strong.color),
                    }
                }),
            Space::new().width(20),
            Space::new().width(Fill),
            text(APP_NAME).color(Color::WHITE).size(26),
            Space::new().width(Fill),
            image_dropdown(
                Sites::iter().collect(),
                current_site.clone().map(Into::into),
                dropdown_open,
                |val| Message::Interaction(Interaction::SiteDropdown(val)),
                "Select a site"
            ),
            Space::new().width(10),
            button( text("Settings").color(Color::WHITE) )
                .on_press(Message::ToggleSettings),
            Space::new().width(20),
        ]
            .width(Fill)
            .align_y(Alignment::Center)
    )
        .width(Fill)
        .padding([20, 0])
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            Style {
                background: Some(palette.background.weaker.color.into()),
                text_color: Some(palette.background.weaker.text),
                border: Border {
                    width: 3.0,
                    radius: 0.into(),
                    color: palette.background.weak.color,
                },
                ..Style::default()
            }
        })
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

        container(image_viewer)
            .center_x(Fill).center_y(Fill)
            .into()
    }
    else if appdata.current_site.is_some() {
        container(
            text("Fetch an image!").size(24)
        )
        .center_x(Fill).center_y(Fill)
        .into()
    }
    else {
        container(
            text("Select a site!").size(24)
        )
        .center_x(Fill).center_y(Fill)
        .into()
    }
}
