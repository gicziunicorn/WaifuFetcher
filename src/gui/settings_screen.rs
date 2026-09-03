#![allow(unused)]

use iced::{
    Border, Color, Element, Length::{self, Fill}, Theme, alignment::{Horizontal, Vertical}, mouse::Interaction::Text, widget::{Container, button, column, container, mouse_area, row, stack, text, opaque}
};
use iced::widget::container::Style;
use iced_fonts::bootstrap;

use crate::app::{AppData, Message};


pub fn settings_screen(appstate: &AppData) -> Element<'_, Message> {
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

    let main_screen_content = container(
        column![
            top,
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

    // the solid background menu that has the contents
    let main_screen = container(
        opaque( main_screen_content )  // opaque makes the contents consume the clicks, otherwise the overlay's mousearea would trigger
    )
        .width(Fill)
        .padding(200);

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
        .on_release(Message::ToggleSettings);

    // the final layout
    // stack is needed here for opaque to work
    let screen = stack![
        overlay,
        main_screen,
    ];

    screen.into()
}