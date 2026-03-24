use iced::widget::{button, container, row, text};
use iced::{Element, Renderer, Theme};

use crate::color::MercuryColors;

/// The state of a guarded button's safety mechanism.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, strum::Display, strum::EnumString,
)]
pub enum GuardState {
    /// Safety is on — button shows guarded appearance.
    #[default]
    Armed,
    /// Safety removed — button shows confirmation prompt.
    Confirming,
}

/// Creates a guarded button — a two-phase button requiring confirmation.
///
/// Inspired by the safety pin on the Mercury hand controller: a deliberate
/// two-step action to prevent accidental activation of critical commands.
///
/// - `Armed`: dimmed button showing "SAFE" badge. Click enters `Confirming`.
/// - `Confirming`: bright danger-colored button showing "CONFIRM: {label}".
///   Click fires `on_execute`. A re-arm button is also shown.
///
/// The guard state lives in the application state, not in the widget.
#[must_use]
pub fn guarded_button<'a, Message: Clone + 'a>(
    label: &str,
    guard_state: GuardState,
    on_confirm: Message,
    on_execute: Message,
    on_arm: Message,
) -> Element<'a, Message, Theme, Renderer> {
    let colors = MercuryColors::default();

    let font_size = 11;

    match guard_state {
        GuardState::Armed => {
            let btn = button(
                text(format!("SAFE  {}", label.to_uppercase()))
                    .size(font_size)
                    .color(colors.tick_mark),
            )
            .on_press(on_confirm)
            .padding(8)
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(colors.panel_bg)),
                text_color: colors.tick_mark,
                border: iced::Border {
                    color: colors.alert,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..button::Style::default()
            });

            container(btn).into()
        }
        GuardState::Confirming => {
            let confirm_btn = button(
                text(format!("CONFIRM: {}", label.to_uppercase()))
                    .size(font_size)
                    .color(colors.text),
            )
            .on_press(on_execute)
            .padding(8)
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(colors.alert)),
                text_color: colors.text,
                border: iced::Border {
                    color: colors.alert,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..button::Style::default()
            });

            let cancel_btn = button(
                text("SAFE").size(font_size).color(colors.tick_mark),
            )
            .on_press(on_arm)
            .padding(8)
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(colors.bezel)),
                text_color: colors.tick_mark,
                border: iced::Border {
                    color: colors.bezel,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..button::Style::default()
            });

            container(
                row![confirm_btn, cancel_btn]
                    .spacing(4)
                    .align_y(iced::Alignment::Center),
            )
            .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn guard_state_roundtrip() {
        for state in [GuardState::Armed, GuardState::Confirming] {
            let s = state.to_string();
            let parsed = GuardState::from_str(&s).unwrap();
            assert_eq!(parsed, state);
        }
    }

    #[test]
    fn guard_state_from_str_invalid() {
        assert!(GuardState::from_str("garbage").is_err());
    }

    #[test]
    fn guard_state_default_is_armed() {
        assert_eq!(GuardState::default(), GuardState::Armed);
    }
}
