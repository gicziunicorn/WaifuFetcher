#![warn(unused)]
use iced::{
    Alignment, Border, Color, Element, Length::{self, Fill, Shrink}, Padding, Theme, gradient::Linear as LinearGradient, padding, theme::palette::{darken, deviate},
};
use iced::widget::{
    button, column, container, mouse_area, opaque, row, stack, text, toggler, text_input,
    text::LineHeight, Space,
    container::Style as ContainerStyle,
    button::Style as ButtonStyle,
    text::Style as TextStyle,
    toggler::Style as TogglerStyle,
    toggler::Status as TogglerStatus,
};
use iced_fonts::bootstrap::{floppytwo_fill, foldertwo, x};

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
    let default_toggler_style = |theme: &Theme, status: TogglerStatus| {
        let palette = theme.extended_palette();

        let inactive_style = TogglerStyle {
            background: palette.background.weakest.color.into(),
            background_border_width: 1.5,
            background_border_color: palette.background.strong.color.into(),
            foreground: palette.background.strong.color.into(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::WHITE,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.175,
        };

        let disabled_inactive_style = TogglerStyle {
            background: darken(palette.background.weakest.color, 0.07).into(),
            background_border_width: 1.5,
            background_border_color: palette.background.strong.color.into(),
            foreground: darken(palette.background.strong.color, 0.07).into(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::WHITE,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.175,
        };

        let active_style = TogglerStyle {
            background: palette.success.base.color.into(),
            background_border_width: 1.5,
            background_border_color: palette.background.weakest.color.into(),
            foreground: palette.background.weakest.color.into(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::WHITE,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.175,
        };

        let disabled_active_style = TogglerStyle {
            background: darken(palette.success.base.color, 0.07).into(),
            background_border_width: 1.5,
            background_border_color: palette.background.weakest.color.into(),
            foreground: darken(palette.background.weakest.color, 0.07).into(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::WHITE,
            text_color: None,
            border_radius: None,
            padding_ratio: 0.175,
        };

        match status {
            TogglerStatus::Active{is_toggled} |
            TogglerStatus::Hovered{is_toggled} => {
                match is_toggled {
                    false => inactive_style,
                    true => active_style,
                }
            },
            TogglerStatus::Disabled{is_toggled} => {
                match is_toggled {
                    false => disabled_inactive_style,
                    true => disabled_active_style,
                }
            }
        }

    };

    let description_text_style = |theme: &Theme| {
        let color = deviate(theme.extended_palette().background.weaker.text, 0.07).scale_alpha(0.75);
        TextStyle { color: Some(color) }
    };

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


        // Values
        row![ text( format!("Image opacity: {:.1}", settings.image_opacity) ) ],
        row![ text( format!("Image scale minimum: {:.3}", settings.image_min_scale) ) ],
        row![ text( format!("Image scale maximum: {:.3}", settings.image_max_scale) ) ],
        row![ text( format!("Image scale step: {:.2}", settings.image_scale_step) ) ],
        // -----------------


        Space::new().height(20),

        // toggler(states.disabled)
        //     .label("disabled")
        //     .on_toggle(Message::Disable)
        //     .style(default_toggler_style),

        // Set auto fetch
        row![
            text("Auto fetch").size(21),
            toggler(settings.auto_fetch)
                .size(24)
                .on_toggle(|v| Message::Interaction(Interaction::ToggleAutoFetch(v)))
                .style(default_toggler_style),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        // -----------------

        Space::new().height(2),
        // Description
        column![
            text("Automatically fetch images from the selected site at set intervals.").size(14)
                .style(description_text_style),
            text("Nekos.best doesn't support this feature.").size(14)
                .style(description_text_style),
        ].padding(padding::left(10)),


        Space::new().height(16),


        // Auto fetch interval
        row![
            text( format!("Auto fetch interval: {}s", settings.auto_fetch_interval as f64/1000.0) ).size(18),
            text( format!("({}ms)", settings.auto_fetch_interval) ).size(18)
                .style(|theme: &Theme| {
                    let color = theme.extended_palette().background.weaker.text.scale_alpha(0.7);
                    TextStyle { color: Some(color) }
                }),
        ].spacing(8),

        Space::new().height(6),

        row![
            text("Change interval:").size(18),
            Space::new().width(6),
            text_input("Enter a number", &states.auto_fetch_int_input)
                .width(60)
                .on_input(|val| Message::Interaction(Interaction::AutoFetchIntervalInput(val)))
                .on_submit(Message::Interaction(Interaction::AutoFetchIntervalSubmit)),
            text("s").size(18),
        ]
        .align_y(Alignment::Center),
        // -----------------

    ].into()
}