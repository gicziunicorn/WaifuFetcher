#![warn(unused)]

use iced::Length::{self, Fill};
use iced::{
    Alignment, Border, Color, Element, Theme,
    widget::{ Space, button, column, row, text, container, Container, stack, image::Viewer, container::Style }
};
use crate::components::dropdown::{image_dropdown};
use crate::app::{AppData, Message};


pub fn main_screen(appdata: &AppData) -> Element<'_, Message> {
    column!(
        navbar(appdata),
        picture_displayer(appdata)
    ).into()
}

pub fn navbar(appdata: &AppData) -> Element<'_, Message> {
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
            button( text("Folder").color(Color::WHITE) )
                .on_press(Message::OpenFileDialog)
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => { button::Style::default().with_background(palette.background.stronger.color) }
                        _ => button::Style::default().with_background(palette.background.strong.color),
                    }
                }),
            Space::new().width(Fill),
            text("Waifu Fetcher").color(Color::WHITE).size(26),
            Space::new().width(Fill),
            image_dropdown(&appdata.sites, appdata.choice.as_ref(), appdata.misc.dropdown_open, Message::DropdownChanged, "Select a site"),
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
    if let Some(handle) = &appdata.image_handle {
        let image = Viewer::new(handle)
            .width(
                if appdata.settings.fill_screen { Length::Fill }
                else { Length::Fixed(appdata.width) }
            )
            .height(
                if appdata.settings.fill_screen { Length::Fill }
                else { Length::Fixed(appdata.height) }
            );
        let _overlay: Container<'_, Message> = container("")
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.6).into()),
                    ..Default::default()
                }
            });
        container(
            //stack!(image, overlay).width(Length::Shrink).height(Length::Shrink)
            stack!(image)
        )
        .center_x(Fill).center_y(Fill)
        .into()
    }
    else {
        container(
            text("Loading...").size(20)
        )
        .center_x(Fill).center_y(Fill)
        .into()
    }
}