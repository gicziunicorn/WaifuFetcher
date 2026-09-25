#![allow(unused)]
use std::path::PathBuf;
use iced::{
    Border, Color, Element, Length::{self, Fill}, Padding, Theme, alignment::{Horizontal, Vertical}, widget::{Container, Text, button, column, container, mouse_area, opaque, row, stack, text}
};
use iced::widget::container::Style;
use iced_fonts::bootstrap;

use crate::app::{AppData, Message, Settings};


pub fn settings_screen(settings: &Settings, focused: bool) -> Element<'_, Message> {
    /*if !appdata.in_settings {
        return container("").into();
    }*/
    let x_icon = bootstrap::x().color(Color::WHITE).size(30).align_x(Horizontal::Center).align_y(Vertical::Center);


    let top = stack![
        container( text("Settings").size(26) )
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center),

        container(
            button(x_icon)
                .padding(2)
                .height(Length::Shrink)
                .width(Length::Shrink)
                .on_press(Message::ToggleSettings)
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    let bg = match status {
                        button::Status::Active => palette.background.stronger.color,
                        _ => palette.background.strong.color,
                    };
                    button::Style {
                        background: Some(bg.into()),
                        border: Border {
                            color: Color::WHITE,
                            width: 0.0,
                            radius: 4.into()
                        },
                        ..button::Style::default()
                    }
                })
        )
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center),
    ];
    let center = center(&settings.save_path);

    let main_screen_content = container(
        column![
            top,
            center,
        ]
    )
        .width(Fill)
        .height(Fill)
        .padding(20)
        // using move so this function takes ownership of focused
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let border_color = match focused {
                true => palette.background.stronger.color,
                false => palette.background.weak.color,
            };
            Style {
                background: Some(palette.background.weaker.color.into()),
                text_color: Some(palette.background.weaker.text),
                border: Border {
                    width: 3.0,
                    radius: 15.into(),
                    color: border_color,
                },
                ..Style::default()
            }
        });

    // the solid background menu that has the contents
    let main_screen = container(
     // opaque makes the contents consume the clicks, otherwise the overlay's mousearea would trigger
        opaque(
            mouse_area( main_screen_content )
                .on_enter(Message::SettingsMenuFocused(true))
        )
    )
        .width(Fill)
        .padding(Padding::from([100, 200]));

    // the darkening overlay for the popup menu
    let overlay = mouse_area(
        container("")
            .center(Fill)
            .style(|_theme: &Theme| {
                container::Style {
                    background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.67).into()),
                    ..Default::default()
                }
            })
    )
        .interaction(iced::mouse::Interaction::Idle)
        .on_release(Message::ToggleSettings)
        .on_enter(Message::SettingsMenuFocused(false));


    // the final layout
    // stack is needed here for opaque to work
    stack![
        overlay,
        main_screen,
    ].into()
}


fn center(save_path: &PathBuf) -> Element<'_, Message> {
    column![
        row![
            text("Save folder").size(20),
            button( text("Folder").color(Color::WHITE) )
                .on_press(Message::OpenFileDialog)
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => { button::Style::default().with_background(palette.background.stronger.color) }
                        _ => button::Style::default().with_background(palette.background.strong.color),
                    }
                }),
            text( format!("current folder: {}", save_path.to_string_lossy().into_owned()) )
        ]
    ].into()
}