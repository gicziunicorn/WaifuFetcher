use iced::widget::container::Style;
use iced::widget::{Container, Scrollable, button, column, container, image, row, text};
use iced::{Alignment, Border, Color, Element, Length, Theme};
use iced_aw::widget::drop_down::DropDown;

#[derive(Debug, Clone)]
pub struct DropdownItem<T> {
    pub value: T,
    pub image_path: &'static str,
}

/// The internal actions of the dropdown, fully managed here.
#[derive(Debug, Clone)]
pub enum DropdownAction<T> {
    Toggle(bool),
    Select(T),
}

/// Purely functional view that handles its own open/close mapping internally
pub fn image_dropdown<'a, Message, T>(
    items: &'a [DropdownItem<T>],
    selected_value: Option<&T>,
    is_open: bool,
    callback: impl Fn(DropdownAction<T>) -> Message + 'a,
    placeholder: &'a str,
) -> Element<'a, Message>
where
    T: Clone + PartialEq + 'static + std::fmt::Display,
    Message: Clone + 'a,
{
    // get the current value
    let current_item = selected_value.and_then(|val| items.iter().find(|i| &i.value == val));

    // the button & its content that triggers the dropdown
    let trigger_content = match current_item {
        Some(item) => column![
            row![
                image(item.image_path).width(22).height(22),
                text(format!("{}", item.value))
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
        let option_btn = button(
            column![
                row![
                    image(item.image_path).width(22).height(22),
                    text(format!("{}", item.value))
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
        .on_press(callback(DropdownAction::Select(item.value.clone())));

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