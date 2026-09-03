#![warn(unused)]

use iced::{
    Border, Color, Element, Length::Fill, Theme, widget::{Container, button, column, container, mouse_area, text}
};
use iced::widget::container::Style;
use crate::{app::{AppData, Message}};


pub fn settings_screen(appstate: &AppData) -> Element<'_, Message> {
    let main_screen = container(
        column![
            text("Settings"),
            button( text("Close").color(Color::WHITE) )
                .on_press(Message::ToggleSettings),
        ]
    )
    .width(Fill)
    .height(Fill)
    .padding(20)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        Style {
            background: Some(palette.background.weaker.color.into()),
            text_color: Some(palette.background.weaker.text),
            border: Border {
                width: 3.0,
                radius: 15.into(),
                color: palette.background.weak.color,
            },
            ..Style::default()
        }
    });

    let overlay_bg: Container<'_, Message> = container(main_screen)
        .center(Fill)
        .padding(200)
        .style(|_theme: &Theme| {
            container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.67).into()),
                ..Default::default()
            }
        });
    let overlay = mouse_area(overlay_bg)
        .interaction(iced::mouse::Interaction::Idle)
        .on_release(Message::ToggleSettings);

    overlay.into()
}