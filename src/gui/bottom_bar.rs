#![warn(unused)]
use iced::Length::Fill;
use iced::widget::Space;
use iced::{
    Alignment, Border, Element, Theme,
    widget::{ row, text, container, container::Style }
};
use crate::app::AppData;
use crate::update::Message;



pub fn bottom_bar(appdata: &AppData) -> Element<'_, Message> {
    container(
        row![
            if let Some(site) = &appdata.current_site {
                let rl_rem = site.get_ratelimit_remaining();

                text(format!("Remaining rate limit: {}", rl_rem)).size(14)
            }
            else { text("") },

            Space::new().width(20),

            if let Some(site) = &appdata.current_site {
                let rl_rst = site.get_ratelimit_reset()
                    .map(|date| date.to_string())
                    .unwrap_or(String::from("-"));

                text(format!("Rate limit reset: {}", rl_rst)).size(14)
            }
            else { text("") }
        ]
        .width(Fill)
        .align_y(Alignment::Center)
    )
    .width(Fill)
    .padding([4, 12])
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