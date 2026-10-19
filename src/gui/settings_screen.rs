#![allow(unused)]
use std::path::PathBuf;
use iced::{
    Border, Color, Element, Gradient, Length::{self, Fill}, Padding, Radians, Theme, alignment::{Horizontal, Vertical}, gradient::Linear as LinearGradient, widget::{Container, Space, Text, button, column, container, mouse_area, opaque, row, stack, text}
};
use iced::widget::container::Style;
use iced_fonts::bootstrap;

use crate::app::{AppData, Message, Settings};


pub fn settings_screen(settings: &Settings, focused: bool) -> Element<'_, Message> {
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
    let hr: Container<'_, Message> = container("")
        .height(2)
        .width(Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();
            let gradient = LinearGradient::new(90)
                .add_stop(0.0, Color::TRANSPARENT)
                .add_stop(0.5, palette.background.strong.color)
                .add_stop(1.0, Color::TRANSPARENT);

            container::Style {
                background: Some(gradient.into()),
                ..Default::default()
            }
        })
        .into();
    let center = center(&settings);

    let main_screen_content = container(
        column![
            top,
            Space::new().height(20),
            hr,
            Space::new().height(40),
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


fn center(settings: &Settings) -> Element<'_, Message> {
    column![
        row![
            text("Save folder").size(20),
            button( text("Folder").color(Color::WHITE) )
                .on_press(Message::PickFolder)
                .style(|theme: &Theme, status| {
                    let palette = theme.extended_palette();
                    match status {
                        button::Status::Active => { button::Style::default().with_background(palette.background.stronger.color) }
                        _ => button::Style::default().with_background(palette.background.strong.color),
                    }
                }),
            text( format!("current folder: {}", settings.save_path.to_string_lossy().into_owned()) )
        ],
        row![
            text( format!("Image opacity: {:.1}", settings.image_opacity) )
        ],
        row![
            text( format!("Image scale minimum: {:.3}", settings.image_min_scale) )
        ],
        row![
            text( format!("Image scale maximum: {:.3}", settings.image_max_scale) )
        ],
        row![
            text( format!("Image scale step: {:.2}", settings.image_scale_step) )
        ],

        button( text("Save settings").color(Color::WHITE) )
            .on_press(Message::SaveSettings)
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                match status {
                    button::Status::Active => { button::Style::default().with_background(palette.background.stronger.color) }
                    _ => button::Style::default().with_background(palette.background.strong.color),
                }
            }),
    ].into()
}