use iced::widget::{button, container, text};
use iced::{Element, Renderer, Theme};

use crate::color::MercuryColors;

/// The state of an async action button.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, strum::Display, strum::EnumString,
)]
pub enum ActionState {
    /// Ready to fire — gray.
    #[default]
    Idle,
    /// Operation in flight — amber.
    Pending,
    /// Operation completed successfully — green.
    Succeeded,
    /// Operation failed — red.
    Failed,
}

/// Creates a tri-state button representing a fallible async operation.
///
/// Visually resembles an indicator tile but is clickable. The button transitions
/// through states: gray (idle) → amber (pending) → green/red (result).
///
/// - Only clickable when `Idle`.
/// - The app manages state transitions: click fires `on_action`, then the app
///   sets `Pending`, and eventually `Succeeded`/`Failed` when the async result
///   arrives. After a hold duration the app resets to `Idle`.
#[must_use]
pub fn async_action_button<'a, Message: Clone + 'a>(
    label: &str,
    state: ActionState,
    on_action: Message,
) -> Element<'a, Message, Theme, Renderer> {
    let colors = MercuryColors::default();

    let (bg_color, text_color, is_clickable) = match state {
        ActionState::Idle => (colors.off, colors.text, true),
        ActionState::Pending => (colors.caution, colors.background, false),
        ActionState::Succeeded => (colors.nominal, colors.background, false),
        ActionState::Failed => (colors.alert, colors.text, false),
    };

    let label_text = text(label.to_uppercase())
        .size(11)
        .color(text_color);

    let mut btn = button(
        container(label_text)
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fixed(32.0)),
    )
    .padding(8)
    .width(iced::Length::Fill)
    .style(move |_theme: &Theme, _status| button::Style {
        background: Some(iced::Background::Color(bg_color)),
        text_color,
        border: iced::Border {
            color: MercuryColors::with_alpha(bg_color, 0.8),
            width: 1.5,
            radius: 2.0.into(),
        },
        ..button::Style::default()
    });

    if is_clickable {
        btn = btn.on_press(on_action);
    }

    btn.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn action_state_roundtrip() {
        for state in [
            ActionState::Idle,
            ActionState::Pending,
            ActionState::Succeeded,
            ActionState::Failed,
        ] {
            let s = state.to_string();
            let parsed = ActionState::from_str(&s).unwrap();
            assert_eq!(parsed, state);
        }
    }

    #[test]
    fn action_state_from_str_invalid() {
        assert!(ActionState::from_str("garbage").is_err());
    }

    #[test]
    fn action_state_default_is_idle() {
        assert_eq!(ActionState::default(), ActionState::Idle);
    }
}
