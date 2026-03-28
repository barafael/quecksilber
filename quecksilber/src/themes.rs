use iced::theme::Palette;
use iced::{Color, Theme};

pub fn cockpit() -> Theme {
    Theme::custom(
        "Cockpit".to_string(),
        Palette {
            primary: Color::BLACK,
            background: Color::WHITE,
            text: Color::WHITE,
            success: Color::WHITE,
            warning: Color::from_rgb(0.9, 0.45, 0.0),
            danger: Color::WHITE,
        },
    )
}
