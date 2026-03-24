use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Event, Frame, Geometry, Path, Stroke};
use iced::{Point, Rectangle, Renderer, Size, Theme};

use crate::anim::Spring;
use crate::color::MercuryColors;
use crate::indicator::IndicatorStatus;

/// A colored zone within a status bar's range.
#[derive(Debug, Clone)]
pub struct Zone {
    /// Start of the zone's range (inclusive).
    pub start: f32,
    /// End of the zone's range (inclusive).
    pub end: f32,
    /// Status level determining the zone's color.
    pub status: IndicatorStatus,
}

/// A horizontal bar showing a value within a range, with colored zones.
///
/// Inspired by the analog meter bands on Mercury spacecraft panels,
/// where colored arcs mark nominal, caution, and danger operating ranges.
/// Here the same concept is linearized into a horizontal bar.
#[derive(Debug, Clone)]
pub struct StatusBar {
    value: f32,
    range_start: f32,
    range_end: f32,
    zones: Vec<Zone>,
    label: String,
    unit: String,
    colors: MercuryColors,
}

impl StatusBar {
    /// Creates a new status bar.
    #[must_use]
    pub fn new(
        label: impl Into<String>,
        value: f32,
        range_start: f32,
        range_end: f32,
    ) -> Self {
        Self {
            value,
            range_start,
            range_end,
            zones: Vec::new(),
            label: label.into(),
            unit: String::new(),
            colors: MercuryColors::default(),
        }
    }

    /// Adds a colored zone to the bar.
    #[must_use]
    pub fn zone(mut self, start: f32, end: f32, status: IndicatorStatus) -> Self {
        self.zones.push(Zone { start, end, status });
        self
    }

    /// Sets the unit label (displayed after the numeric value).
    #[must_use]
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    /// Sets a custom color palette.
    #[must_use]
    pub fn colors(mut self, colors: MercuryColors) -> Self {
        self.colors = colors;
        self
    }

    fn normalize(&self, v: f32) -> f32 {
        let range = self.range_end - self.range_start;
        if range.abs() < f32::EPSILON {
            return 0.0;
        }
        ((v - self.range_start) / range).clamp(0.0, 1.0)
    }
}

/// Internal state for the status bar's marker animation.
#[derive(Debug)]
pub struct StatusBarState {
    marker_spring: Spring,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            // Overdamped spring for smooth slide, no bounce
            marker_spring: Spring::with_params(0.0, 120.0, 1.5),
        }
    }
}

impl canvas::Program<(), Theme, Renderer> for StatusBar {
    type State = StatusBarState;

    fn update(
        &self,
        state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<()>> {
        let target = self.normalize(self.value);
        state.marker_spring.set_target(target);

        if !state.marker_spring.is_settled() {
            state.marker_spring.tick(1.0 / 60.0);
            Some(canvas::Action::request_redraw())
        } else {
            None
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let mut frame = Frame::new(renderer, bounds.size());

        let padding_x = 8.0;
        let label_height = 16.0;
        let bar_height = 12.0;
        let bar_y = label_height + 4.0;
        let bar_width = bounds.width - padding_x * 2.0;

        // Background
        let bg = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.fill(&bg, self.colors.panel_bg);

        // Label (left)
        frame.fill_text(canvas::Text {
            content: self.label.clone(),
            position: Point::new(padding_x, 2.0),
            color: self.colors.text,
            size: iced::Pixels(11.0),
            align_x: iced::alignment::Horizontal::Left.into(),
            align_y: iced::alignment::Vertical::Top,
            ..canvas::Text::default()
        });

        // Numeric value (right)
        let value_text = if self.unit.is_empty() {
            format!("{:.0}", self.value)
        } else {
            format!("{:.0} {}", self.value, self.unit)
        };
        frame.fill_text(canvas::Text {
            content: value_text,
            position: Point::new(bounds.width - padding_x, 2.0),
            color: self.colors.text,
            size: iced::Pixels(11.0),
            align_x: iced::alignment::Horizontal::Right.into(),
            align_y: iced::alignment::Vertical::Top,
            ..canvas::Text::default()
        });

        // Bar track
        let track = Path::rectangle(
            Point::new(padding_x, bar_y),
            Size::new(bar_width, bar_height),
        );
        frame.fill(&track, self.colors.bezel);

        // Zone segments
        for zone in &self.zones {
            let x_start = self.normalize(zone.start) * bar_width + padding_x;
            let x_end = self.normalize(zone.end) * bar_width + padding_x;
            let zone_width = x_end - x_start;
            if zone_width > 0.0 {
                let zone_color = self.colors.status_color(zone.status);
                // Draw zone dimmed
                let dimmed = MercuryColors::with_alpha(zone_color, 0.35);
                let zone_path = Path::rectangle(
                    Point::new(x_start, bar_y),
                    Size::new(zone_width, bar_height),
                );
                frame.fill(&zone_path, dimmed);
            }
        }

        // Marker (animated)
        let marker_x = state.marker_spring.value() * bar_width + padding_x;

        // Determine zone at current value for marker color
        let marker_color = self.zone_at_value(self.value);

        // Marker line
        let marker = Path::line(
            Point::new(marker_x, bar_y - 2.0),
            Point::new(marker_x, bar_y + bar_height + 2.0),
        );
        frame.stroke(
            &marker,
            Stroke::default().with_color(marker_color).with_width(2.0),
        );

        // Small triangle above
        let tri = Path::new(|builder| {
            builder.move_to(Point::new(marker_x - 4.0, bar_y - 2.0));
            builder.line_to(Point::new(marker_x + 4.0, bar_y - 2.0));
            builder.line_to(Point::new(marker_x, bar_y + 1.0));
            builder.close();
        });
        frame.fill(&tri, marker_color);

        // Outer border
        let border = Path::rectangle(Point::ORIGIN, bounds.size());
        frame.stroke(
            &border,
            Stroke::default()
                .with_color(self.colors.bezel)
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }
}

impl StatusBar {
    fn zone_at_value(&self, value: f32) -> iced::Color {
        for zone in &self.zones {
            if value >= zone.start && value <= zone.end {
                return self.colors.status_color(zone.status);
            }
        }
        self.colors.needle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_at_boundaries() {
        let bar = StatusBar::new("test", 0.0, 0.0, 100.0);
        assert!((bar.normalize(0.0) - 0.0).abs() < f32::EPSILON);
        assert!((bar.normalize(100.0) - 1.0).abs() < f32::EPSILON);
        assert!((bar.normalize(50.0) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_clamps() {
        let bar = StatusBar::new("test", 0.0, 0.0, 100.0);
        assert!((bar.normalize(-10.0) - 0.0).abs() < f32::EPSILON);
        assert!((bar.normalize(200.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_zero_range() {
        let bar = StatusBar::new("test", 50.0, 50.0, 50.0);
        assert!((bar.normalize(50.0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn zone_at_value_finds_correct_zone() {
        let bar = StatusBar::new("test", 50.0, 0.0, 100.0)
            .zone(0.0, 60.0, IndicatorStatus::Nominal)
            .zone(60.0, 80.0, IndicatorStatus::Caution)
            .zone(80.0, 100.0, IndicatorStatus::Alert);

        let colors = MercuryColors::default();
        assert_eq!(bar.zone_at_value(30.0), colors.nominal);
        assert_eq!(bar.zone_at_value(70.0), colors.caution);
        assert_eq!(bar.zone_at_value(90.0), colors.alert);
    }

    #[test]
    fn zone_at_value_outside_zones_returns_needle() {
        let bar = StatusBar::new("test", 50.0, 0.0, 100.0)
            .zone(20.0, 40.0, IndicatorStatus::Nominal);

        let colors = MercuryColors::default();
        assert_eq!(bar.zone_at_value(10.0), colors.needle);
    }
}
