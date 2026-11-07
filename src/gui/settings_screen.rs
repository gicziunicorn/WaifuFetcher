#![allow(unused)]
use iced::{
    Alignment, Border, Color, Element, Length::{self, Fill, Shrink}, Padding, Theme, alignment::{Horizontal, Vertical}, gradient::Linear as LinearGradient, theme::palette::deviate, widget::{space, text::LineHeight, text_input, toggler}
};
use iced::widget::{
    Space, button, column, container, mouse_area, opaque, row, stack, text,
    container::Style as ContainerStyle, button::Style as ButtonStyle, text::Style as TextStyle, toggler::Style as TogglerStyle,
};
use iced_fonts::bootstrap::{self, floppy, floppytwo_fill, foldertwo, x};

use crate::{app::{Settings, States}, update::{Interaction, Message}};


pub fn settings_screen<'a>(settings: &'a Settings, states: &'a States) -> Element<'a, Message> {
    // -- Parts of the menu --
    let top = row![
        // Save button
        container(
            button(
                row![
                    floppytwo_fill().line_height(LineHeight::Relative(1.0)).size(18)
                        .style(|theme: &Theme| {
                            let color = deviate(theme.extended_palette().background.strong.text, -0.03);
                            TextStyle { color: Some(color) }
                        }),
                    text("Save settings").line_height(LineHeight::Relative(1.0))
                        .style(|theme: &Theme| {
                            let color = deviate(theme.extended_palette().background.strong.text, -0.03);
                            TextStyle { color: Some(color) }
                        })
                ]
                .spacing(8)
                .align_y(Alignment::Center)
            )
            .padding([8, 10])
            .width(Shrink)
            .height(Shrink)
            .on_press(Message::SaveSettings)
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
                        radius: 8.into()
                    },
                    ..button::Style::default()
                }
            }),
        )
            .width(Length::FillPortion(1))
            .height(Length::Shrink)
            .align_x(Alignment::Start),

        // Title
        container( text("Settings").size(26) )
            .width(Length::FillPortion(1))
            .height(Length::Shrink)
            .align_x(Alignment::Center),

        // Close button
        container(
            button(
                x().line_height(LineHeight::Relative(1.0)).size(30)
                    .style(|theme: &Theme| {
                        let color = deviate(theme.extended_palette().background.strong.text, -0.03);
                        TextStyle { color: Some(color) }
                    })
            )
            .padding(2)
            .width(Shrink)
            .height(Shrink)
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
                        radius: 8.into()
                    },
                    ..button::Style::default()
                }
            })
        )
            .width(Length::FillPortion(1))
            .height(Length::Shrink)
            .align_x(Alignment::End),
    ]
        .height(Length::Shrink)
        .align_y(Alignment::Center);

    let hr: Element<'_, Message> = container("")
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

    let center = center(settings, states);
    // ------

    // The contents of the solid background menu
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
            let border_color = match states.settings_menu_focused {
                true => palette.background.stronger.color,
                false => palette.background.weak.color,
            };
            ContainerStyle {
                background: Some(palette.background.weaker.color.into()),
                border: Border {
                    width: 3.0,
                    radius: 15.into(),
                    color: border_color,
                },
                ..ContainerStyle::default()
            }
        });

    // The solid background menu
    let main_screen = container(
     // opaque makes the contents consume the clicks, otherwise the overlay's mousearea would trigger
        opaque(
            mouse_area( main_screen_content )
                .on_enter(Message::SettingsMenuFocused(true))
        )
    )
        .width(Fill)
        .padding(Padding::from([100, 200]));

    // The darkened background overlay for the popup menu
    let overlay = mouse_area(
        container("")
            .center(Fill)
            .style(|_theme: &Theme| {
                ContainerStyle {
                    background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.67).into()),
                    ..Default::default()
                }
            })
    )
        .interaction(iced::mouse::Interaction::Idle) //cursor
        .on_release(Message::ToggleSettings)
        .on_enter(Message::SettingsMenuFocused(false));


    // The final layout
    // using stack instead of a simple container is needed here for opaque to work
    stack![
        overlay,
        main_screen,
    ].into()
}


fn center<'a>(settings: &'a Settings, states: &'a States) -> Element<'a, Message> {
    column![
        // Image save folder
        row![
            row![
                text("Image save folder:")
                    .line_height(LineHeight::Relative(1.0))
                    .size(20)
                    .style(|theme: &Theme| {
                        let color = theme.extended_palette().background.weaker.text;
                        TextStyle { color: Some(color) }
                    }),

                text( settings.image_save_path.to_string_lossy().into_owned() )
                    .line_height(LineHeight::Relative(1.0))
                    .size(18)
                    .style(|theme: &Theme| {
                        let color = deviate(theme.extended_palette().background.weaker.text, 0.05);
                        TextStyle { color: Some(color) }
                    })
            ]
            .align_y(Alignment::End).spacing(20),

            button(
                row![
                    foldertwo().line_height(LineHeight::Relative(1.0)).size(20)
                        .style(|theme: &Theme| {
                            TextStyle {
                                color: Some(theme.extended_palette().background.strong.text)
                            }
                        }),
                    text("Change folder")
                        .style(|theme: &Theme| {
                            TextStyle {
                                color: Some(theme.extended_palette().background.strong.text)
                            }
                        }),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .on_press(Message::PickFolder)
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                let background = match status {
                    button::Status::Active => palette.background.stronger.color,
                    _ => palette.background.strong.color
                };

                ButtonStyle {
                    background: Some(background.into()),
                    border: Border {
                        radius: 6.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
        ]
        .spacing(20)
        .align_y(Alignment::Center),
        // -----------------

        //
        // -----------------


        row![ text( format!("Image opacity: {:.1}", settings.image_opacity) ) ],
        row![ text( format!("Image scale minimum: {:.3}", settings.image_min_scale) ) ],
        row![ text( format!("Image scale maximum: {:.3}", settings.image_max_scale) ) ],
        row![ text( format!("Image scale step: {:.2}", settings.image_scale_step) ) ],

        Space::new().height(20),

        // Set auto fetch
        toggler(settings.auto_fetch)
            .label("Auto fetch")
            .on_toggle(|val| Message::Interaction(Interaction::ToggleAutoFetch(val))),

        // Auto fetch interval
        text( format!("int: {}", settings.auto_fetch_interval.to_string()) ),
        row![
            text("Auto fetch interval:"),
            text_input("Enter a number", &states.auto_fetch_int_input)
                .on_input(|val| Message::Interaction(Interaction::AutoFetchIntervalInput(val)))
                .on_submit(Message::Interaction(Interaction::AutoFetchIntervalSubmit)),
        ],

    ].into()
}