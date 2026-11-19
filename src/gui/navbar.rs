#![warn(unused)]
use strum::IntoEnumIterator;
use iced::Length::Fill;
use iced::{
    Alignment, Border, Color, Element, Theme,
    widget::{ Space, button, row, text, container, container::Style }
};
use waifu_api::sites::{Site, Sites};

use crate::{
    APP_NAME,
    components::dropdown::image_dropdown,
    update::{Interaction, Message}
};


pub fn navbar(current_site: &Option<Site>, dropdown_open: bool, disable_fetch: bool) -> Element<'_, Message> {
    container(
        row![
            Space::new().width(20),
            button( text("Fetch").color(Color::WHITE) )
                .on_press_maybe(match disable_fetch {
                    false => Some(Message::FetchImage),
                    true => None,
                })
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