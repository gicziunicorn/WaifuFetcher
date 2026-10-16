use std::slice::Iter;
use iced::widget::container::Style;
use iced::widget::{Container, Scrollable, button, column, container, image, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_aw::widget::drop_down::DropDown;
use strum::IntoEnumIterator;
use waifu_api::site::Sites;

use crate::ImagePath;


/// The internal actions of the dropdown, that should be handled
#[derive(Debug, Clone)]
pub enum DropdownAction<T> {
    Toggle(bool),
    Select(T),
}

/// Dropdown with images for each element
pub fn image_dropdown<'a, Message, I>(
    mut items: I,
    selected_value: Option<&I::Item>,
    is_open: bool,
    callback: impl Fn(DropdownAction<I::Item>) -> Message + 'a,
    placeholder: &'a str,
) -> Element<'a, Message>
where
    I: Iterator,
    I::Item: Clone + PartialEq + 'static + std::fmt::Display,
    Message: Clone + 'a,
{
    // get the current value
    let current_item = selected_value.and_then(|val| items.find(|i| i == val));

    // the button & its content that triggers the dropdown
    let trigger_content = match current_item {
        Some(item) => column![
            row![
                image( format!("assets/{}.ico", item.to_string().to_lowercase()) ).width(22).height(22),
                text( format!("{}", item.to_string()) )
            ].align_y(Alignment::Center).spacing(10).width(Length::Shrink),
        ].align_x(Alignment::Center).width(Length::Fill),
        None => column![text(placeholder)].align_x(Alignment::Center),
    };

    let trigger_button = button(trigger_content)
        .width(Length::Fixed(140.0))
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            let bg_color = match status {
                button::Status::Active => palette.background.stronger.color,
                _ => palette.background.strong.color,
            };

            button::Style {
                background: Some(bg_color.into()),
                text_color: palette.primary.strong.text,
                border: Border {
                    width: 1.0,
                    radius: 5.into(),
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                },
                ..button::Style::default()
            }
        })
        .on_press(callback(DropdownAction::Toggle(!is_open)));

    // the dropdown menu content
    let mut menu_column = column![].width(Length::Fill);

    for item in items {
        let image_path = format!("assets/{}.ico", item.to_string().to_lowercase());

        let option_btn = button(
            column![
                row![
                    image(image_path).width(22).height(22),
                    text(format!("{}", item.to_string()))
                ].align_y(Alignment::Center).spacing(10).width(Length::Shrink),
            ].align_x(Alignment::Center).width(Length::Fill)
        )
        .width(Length::Fill)
        .padding([5, 10])
        .style(|theme: &Theme, status| {
            let palette = theme.extended_palette();
            let bg_color = match status {
                button::Status::Active => palette.background.strongest.color,
                _ => palette.background.strong.color,
            };

            button::Style {
                background: Some(bg_color.into()),
                text_color: palette.primary.strong.text,
                border: Border {
                    width: 1.0,
                    radius: 0.into(),
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                },
                ..button::Style::default()
            }
        })
        .on_press(callback(DropdownAction::Select(item.clone())));

        menu_column = menu_column.push(option_btn);
    }

    let menu_content = container( Scrollable::new(menu_column) )
    .max_height(250)
    .width(Length::Fixed(120.0))
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        Style {
            background: Some(palette.background.weaker.color.into()),
            text_color: Some(palette.background.weaker.text),
            border: Border {
                width: 2.0,
                radius: 0.into(),
                color: Color::from_rgb(0.5, 0.5, 0.5),
            },
            ..Style::default()
        }
    });

    DropDown::new(trigger_button, menu_content, is_open).into()
}