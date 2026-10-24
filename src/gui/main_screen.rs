#![warn(unused)]
use iced::widget::Column;
use rand::{Rng, rng};
use strum::IntoEnumIterator;
use iced::Length::{self, Fill};
use iced::{
    Alignment, Border, Color, Element, Theme,
    widget::{ Space, button, column, row, text, container, Container, stack, container::Style }
};
use waifu_api::site::{Site, Sites};
use crate::APP_NAME;
use crate::components::image_viewer::Viewer;
use crate::components::dropdown::image_dropdown;
use crate::app::{AppData, Message, Notification};


pub fn main_screen(appdata: &AppData) -> Element<'_, Message> {
    column!(
        navbar(
            &appdata.current_site,
            appdata.states.dropdown_open,
        ),
        stack!(
            picture_displayer(appdata),
            notifications(&appdata.notifications)
        ),
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
            button( text("Error").color(Color::WHITE) )
                .on_press(Message::Notify("Test error".to_string()))
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => { button::Style::default().with_background(palette.background.stronger.color) }
                        _ => button::Style::default().with_background(palette.background.strong.color),
                    }
                }),
            Space::new().width(Fill),
            text(APP_NAME).color(Color::WHITE).size(26),
            Space::new().width(Fill),
            image_dropdown(
                Sites::iter().collect(),
                current_site.clone().map(Into::into),
                dropdown_open,
                Message::DropdownChanged,
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
        let image = Viewer::new(
            &image.handle,
            Fill,
            Fill,
            appdata.states.zoom,
            appdata.states.image_offset,
            appdata.settings.image_opacity,
            Message::SetViewerScale,
            Message::SetViewerOffset,
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

pub fn notifications(notifications:&Vec<Notification>) -> Element<'_, Message> {
    let notif_col = Column::new().extend(
        notifications.iter().rev()
            .map(|n| notification(n))
    );

    container(
        notif_col.spacing(12)
    )
    .align_bottom(Fill)
    .align_right(Fill)
    .padding(30)
    .into()
}

pub fn notification(notification: &Notification) -> Element<'_, Message> {
    let r = rng().next_u32().to_string();
    let msg = format!("{} - {r}", notification.message);
    container(
        text(msg)
    )
    .padding(15)
    .style(|_| {
        Style {
            background: Some(Color::from_rgba8(0, 0, 0, 0.6).into()),
            text_color: Some(Color::WHITE),
            border: Border {
                width: 0.0,
                radius: 10.into(),
                color: Color::BLACK,
            },
            ..Style::default()
        }
    })
    .into()
}