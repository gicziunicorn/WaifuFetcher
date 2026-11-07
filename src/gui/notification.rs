#![warn(unused)]
use iced::{Border, Element, Color, Length::Fill};
use iced::widget::{Column, container, container::Style as ContainerStyle, text};

use crate::app::Notification;
use crate::update::Message;


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
    container(
        text(&notification.message)
    )
    .padding(15)
    .style(|_| {
        ContainerStyle {
            background: Some(Color::from_rgba8(0, 0, 0, 0.6).into()),
            text_color: Some(Color::WHITE),
            border: Border {
                width: 0.0,
                radius: 10.into(),
                color: Color::BLACK,
            },
            ..ContainerStyle::default()
        }
    })
    .into()
}