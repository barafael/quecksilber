use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Event, Frame, Geometry, Path};
use iced::{Rectangle, Renderer, Theme};

use crate::anim::ColorTransition;
use crate::color::MercuryColors;

/// The status of an indicator light.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, strum::Display, strum::EnumString,
)]
pub enum IndicatorStatus {
    /// Powered off / inactive.
    #[default]
    Off,
    /// System nominal (green).
    Nominal,
    /// Caution / warning (amber).
    Caution,
    /// Alert / critical (red).
    Alert,
}

/// A status light indicator tile, inspired by the backlit annunciator tiles
/// on Mercury mission control consoles.
///
/// Displays a colored status region with a label below it. The status color
/// transitions smoothly between states using exponential decay animation.
#[derive(Debug, Clone)]
pub struct Indicator {
    label: String,
    status: IndicatorStatus,
    colors: MercuryColors,
}

impl Indicator {
    /// Creates a new indicator with the given label and status.
    #[must_use]
    pub fn new(label: impl Into<String>, status: IndicatorStatus) -> Self {
        Self {
            label: label.into(),
            status,
            colors: MercuryColors::default(),
        }
    }

    /// Sets a custom color palette for this indicator.
    #[must_use]
    pub fn colors(mut self, colors: MercuryColors) -> Self {
        self.colors = colors;
        self
    }
}

/// Internal state for the indicator's color animation.
#[derive(Debug)]
pub struct IndicatorState {
    color_transition: ColorTransition,
    last_status: IndicatorStatus,
    initialized: bool,
}

impl Default for IndicatorState {
    fn default() -> Self {
        let colors = MercuryColors::default();
        Self {
            color_transition: ColorTransition::new(colors.off),
            last_status: IndicatorStatus::Off,
            initialized: false,
        }
    }
}

impl canvas::Program<(), Theme, Renderer> for Indicator {
    type State = IndicatorState;

    fn update(
        &self,
        state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<()>> {
        if !state.initialized {
            // First frame: snap to the initial color immediately (no animation).
            let target = self.colors.status_color(self.status);
            state.color_transition.set_target(target);
            state.color_transition.snap();
            state.last_status = self.status;
            state.initialized = true;
            return Some(canvas::Action::request_redraw());
        }

        // Animate toward new target when status changes.
        if self.status != state.last_status {
            let target = self.colors.status_color(self.status);
            state.color_transition.set_target(target);
            state.last_status = self.status;
        }

        // Tick animation forward.
        if !state.color_transition.is_settled() {
            state.color_transition.tick(1.0 / 60.0);
            Some(canvas::Action::request_redraw())
        } else {
            None
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        _renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let current_color = state.color_transition.value();

        let mut frame = Frame::new(_renderer, bounds.size());

        let padding = 4.0;
        let light_height = bounds.height * 0.55;
        let label_area_top = light_height + padding;

        // Panel background
        let bg = Path::rectangle(iced::Point::ORIGIN, bounds.size());
        frame.fill(&bg, self.colors.panel_bg);

        // Status light region
        let light_rect = Rectangle {
            x: padding,
            y: padding,
            width: bounds.width - padding * 2.0,
            height: light_height - padding,
        };

        // Glow effect (slightly larger, transparent)
        if self.status != IndicatorStatus::Off {
            let glow_expand = 2.0;
            let glow_rect = Rectangle {
                x: light_rect.x - glow_expand,
                y: light_rect.y - glow_expand,
                width: light_rect.width + glow_expand * 2.0,
                height: light_rect.height + glow_expand * 2.0,
            };
            let glow_path = Path::rectangle(
                iced::Point::new(glow_rect.x, glow_rect.y),
                iced::Size::new(glow_rect.width, glow_rect.height),
            );
            frame.fill(&glow_path, MercuryColors::with_alpha(current_color, 0.3));
        }

        // Main status light
        let light_path = Path::rectangle(
            iced::Point::new(light_rect.x, light_rect.y),
            iced::Size::new(light_rect.width, light_rect.height),
        );
        frame.fill(&light_path, current_color);

        // Bezel border around the light
        frame.stroke(
            &light_path,
            canvas::Stroke::default()
                .with_color(self.colors.bezel)
                .with_width(1.0),
        );

        // Label text
        let label_center = iced::Point::new(bounds.width / 2.0, label_area_top + padding);
        frame.fill_text(canvas::Text {
            content: self.label.clone(),
            position: label_center,
            color: self.colors.text,
            size: iced::Pixels(11.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Top,
            font: self.colors.font,
            ..canvas::Text::default()
        });

        // Outer bezel
        let outer = Path::rectangle(iced::Point::ORIGIN, bounds.size());
        frame.stroke(
            &outer,
            canvas::Stroke::default()
                .with_color(self.colors.bezel)
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn indicator_status_display_roundtrip() {
        for status in [
            IndicatorStatus::Off,
            IndicatorStatus::Nominal,
            IndicatorStatus::Caution,
            IndicatorStatus::Alert,
        ] {
            let s = status.to_string();
            let parsed = IndicatorStatus::from_str(&s).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn indicator_status_from_str_invalid() {
        let result = IndicatorStatus::from_str("garbage");
        assert!(result.is_err(), "Should fail to parse invalid status");
    }

    #[test]
    fn indicator_status_default_is_off() {
        assert_eq!(IndicatorStatus::default(), IndicatorStatus::Off);
    }

    #[test]
    fn indicator_new_does_not_panic() {
        for status in [
            IndicatorStatus::Off,
            IndicatorStatus::Nominal,
            IndicatorStatus::Caution,
            IndicatorStatus::Alert,
        ] {
            let _ = Indicator::new("TEST", status);
        }
    }
}
