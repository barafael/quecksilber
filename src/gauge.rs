use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Event, Frame, Geometry, Path, Text};
use iced::{Color, Point, Rectangle, Renderer, Theme};

use crate::anim::Spring;
use crate::color::MercuryColors;
use crate::draw;
use crate::indicator::IndicatorStatus;

/// A colored arc zone on the gauge face.
#[derive(Debug, Clone)]
pub struct GaugeArc {
    /// Start of the zone's value range (inclusive).
    pub start: f32,
    /// End of the zone's value range (inclusive).
    pub end: f32,
    /// Status level determining the arc's color.
    pub status: IndicatorStatus,
}

/// A round analog dial gauge with needle, colored arcs, tick marks, and labels.
///
/// Inspired by the round instruments on the Mercury spacecraft main panel —
/// attitude indicators, cabin pressure gauges, voltage meters. The gauge renders
/// a bezel, colored arc zones for nominal/caution/danger ranges, major and minor
/// tick marks with numeric labels, and an animated needle.
///
/// The needle uses a critically damped spring for smooth, realistic motion.
#[derive(Debug, Clone)]
pub struct Gauge {
    value: f32,
    range_start: f32,
    range_end: f32,
    label: String,
    unit: String,
    arcs: Vec<GaugeArc>,
    major_ticks: usize,
    minor_ticks: usize,
    /// Total angular sweep of the gauge in radians (default: 270°).
    sweep_angle: f32,
    /// Start angle in radians (default: 225° = 5π/4, the 7 o'clock position).
    start_angle: f32,
    colors: MercuryColors,
}

impl Gauge {
    /// Creates a new gauge.
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
            label: label.into(),
            unit: String::new(),
            arcs: Vec::new(),
            major_ticks: 5,
            minor_ticks: 4,
            sweep_angle: 270.0_f32.to_radians(),
            start_angle: (225.0_f32).to_radians(),
            colors: MercuryColors::default(),
        }
    }

    /// Sets the unit label shown below the value.
    #[must_use]
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    /// Adds a colored arc zone.
    #[must_use]
    pub fn arc(mut self, start: f32, end: f32, status: IndicatorStatus) -> Self {
        self.arcs.push(GaugeArc { start, end, status });
        self
    }

    /// Sets the number of major tick marks.
    #[must_use]
    pub fn major_ticks(mut self, count: usize) -> Self {
        self.major_ticks = count;
        self
    }

    /// Sets the number of minor ticks between each pair of major ticks.
    #[must_use]
    pub fn minor_ticks(mut self, count: usize) -> Self {
        self.minor_ticks = count;
        self
    }

    /// Sets a custom sweep angle (in degrees).
    #[must_use]
    pub fn sweep_degrees(mut self, degrees: f32) -> Self {
        self.sweep_angle = degrees.to_radians();
        self
    }

    /// Sets a custom start angle (in degrees, 0 = 3 o'clock, counter-clockwise).
    #[must_use]
    pub fn start_degrees(mut self, degrees: f32) -> Self {
        self.start_angle = degrees.to_radians();
        self
    }

    /// Sets a custom color palette.
    #[must_use]
    pub fn colors(mut self, colors: MercuryColors) -> Self {
        self.colors = colors;
        self
    }

    fn value_to_angle(&self, value: f32) -> f32 {
        draw::value_to_angle(
            value,
            self.range_start,
            self.range_end,
            self.start_angle,
            self.sweep_angle,
        )
    }
}

/// Internal state for the gauge's needle animation and geometry cache.
#[derive(Debug)]
pub struct GaugeState {
    cache: Cache<Renderer>,
    needle_spring: Spring,
}

impl Default for GaugeState {
    fn default() -> Self {
        Self {
            cache: Cache::default(),
            // Critically damped spring — needle swings with slight overshoot
            needle_spring: Spring::with_params(0.0, 180.0, 0.85),
        }
    }
}

impl canvas::Program<(), Theme, Renderer> for Gauge {
    type State = GaugeState;

    fn update(
        &self,
        state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<()>> {
        let target_angle = self.value_to_angle(self.value);
        state.needle_spring.set_target(target_angle);

        if !state.needle_spring.is_settled() {
            state.needle_spring.tick(1.0 / 60.0);
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
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = (bounds.width.min(bounds.height) / 2.0) - 8.0;

        // Static geometry (cached — only redrawn when size changes)
        let static_geo = state.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_static(frame, center, radius);
        });

        // Dynamic geometry (needle — redrawn each frame during animation)
        let mut needle_frame = Frame::new(renderer, bounds.size());
        self.draw_needle(&mut needle_frame, center, radius, state.needle_spring.value());

        vec![static_geo, needle_frame.into_geometry()]
    }
}

impl Gauge {
    fn draw_static(&self, frame: &mut Frame<Renderer>, center: Point, radius: f32) {
        let arc_radius = radius * 0.78;
        let arc_width = radius * 0.10;
        let tick_outer = radius * 0.85;
        let tick_inner_major = radius * 0.68;
        let tick_inner_minor = radius * 0.74;
        let label_radius = radius * 0.54;

        // Background circle fill
        let bg_circle = Path::circle(center, radius);
        frame.fill(&bg_circle, self.colors.panel_bg);

        // Bezel
        draw::draw_bezel(frame, center, radius, self.colors.bezel, 2.0);

        // Arc zones
        for arc in &self.arcs {
            let start = self.value_to_angle(arc.start);
            let end = self.value_to_angle(arc.end);
            let zone_color = self.colors.status_color(arc.status);
            let dimmed = MercuryColors::with_alpha(zone_color, 0.7);
            draw::draw_arc_segment(frame, center, arc_radius, start, end, arc_width, dimmed);
        }

        // Major tick marks and numeric labels
        if self.major_ticks > 0 {
            let total_ticks = self.major_ticks;
            for i in 0..=total_ticks {
                let frac = i as f32 / total_ticks as f32;
                let angle = self.start_angle + frac * self.sweep_angle;
                let value = self.range_start + frac * (self.range_end - self.range_start);

                // Major tick
                draw::draw_tick(
                    frame,
                    center,
                    tick_inner_major,
                    tick_outer,
                    angle,
                    self.colors.tick_mark,
                    1.5,
                );

                // Numeric label
                let (sin, cos) = angle.sin_cos();
                let label_pos = Point::new(
                    center.x + cos * label_radius,
                    center.y + sin * label_radius,
                );
                frame.fill_text(Text {
                    content: format!("{value:.0}"),
                    position: label_pos,
                    color: self.colors.tick_mark,
                    size: iced::Pixels(10.0),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    font: self.colors.font,
            ..Text::default()
                });

                // Minor ticks between this major and the next
                if i < total_ticks && self.minor_ticks > 0 {
                    for j in 1..=self.minor_ticks {
                        let minor_frac =
                            frac + (j as f32 / ((self.minor_ticks + 1) as f32 * total_ticks as f32));
                        let minor_angle = self.start_angle + minor_frac * self.sweep_angle;
                        draw::draw_tick(
                            frame,
                            center,
                            tick_inner_minor,
                            tick_outer,
                            minor_angle,
                            self.colors.tick_mark,
                            0.75,
                        );
                    }
                }
            }
        }

        // Label and unit text — placed in the open gap at the bottom of the
        // sweep arc, well below the needle hub so they never overlap the hand.
        let label_y = center.y + radius * 0.45;
        frame.fill_text(Text {
            content: self.label.clone(),
            position: Point::new(center.x, label_y),
            color: self.colors.text,
            size: iced::Pixels(11.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            font: self.colors.font,
            ..Text::default()
        });

        if !self.unit.is_empty() {
            frame.fill_text(Text {
                content: self.unit.clone(),
                position: Point::new(center.x, label_y + 14.0),
                color: self.colors.tick_mark,
                size: iced::Pixels(9.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                font: self.colors.font,
            ..Text::default()
            });
        }
    }

    fn draw_needle(
        &self,
        frame: &mut Frame<Renderer>,
        center: Point,
        radius: f32,
        angle: f32,
    ) {
        let needle_length = radius * 0.75;

        // Determine needle color based on current zone
        let needle_color = self.needle_color_for_angle(angle);

        draw::draw_needle(frame, center, needle_length, angle, needle_color);

        // Hub circle
        let hub = Path::circle(center, 4.0);
        frame.fill(&hub, self.colors.needle);
    }

    fn needle_color_for_angle(&self, angle: f32) -> Color {
        // Reverse-map angle to value to determine zone
        let range = self.range_end - self.range_start;
        if range.abs() < f32::EPSILON || self.sweep_angle.abs() < f32::EPSILON {
            return self.colors.needle;
        }
        let frac = ((angle - self.start_angle) / self.sweep_angle).clamp(0.0, 1.0);
        let value = self.range_start + frac * range;

        for arc in &self.arcs {
            if value >= arc.start && value <= arc.end {
                let zone_color = self.colors.status_color(arc.status);
                // Blend needle white with zone color when in non-nominal zone
                if arc.status != IndicatorStatus::Nominal && arc.status != IndicatorStatus::Off {
                    return Color {
                        r: (self.colors.needle.r + zone_color.r) / 2.0,
                        g: (self.colors.needle.g + zone_color.g) / 2.0,
                        b: (self.colors.needle.b + zone_color.b) / 2.0,
                        a: 1.0,
                    };
                }
                return self.colors.needle;
            }
        }
        self.colors.needle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_value_to_angle_boundaries() {
        let gauge = Gauge::new("test", 0.0, 0.0, 100.0);
        let start = gauge.value_to_angle(0.0);
        let end = gauge.value_to_angle(100.0);
        assert!((start - gauge.start_angle).abs() < f32::EPSILON);
        assert!((end - (gauge.start_angle + gauge.sweep_angle)).abs() < f32::EPSILON);
    }

    #[test]
    fn gauge_value_to_angle_midpoint() {
        let gauge = Gauge::new("test", 50.0, 0.0, 100.0);
        let mid = gauge.value_to_angle(50.0);
        let expected = gauge.start_angle + gauge.sweep_angle / 2.0;
        assert!((mid - expected).abs() < 0.001);
    }

    #[test]
    fn gauge_new_does_not_panic() {
        let _ = Gauge::new("CABIN PRESSURE", 14.7, 0.0, 20.0)
            .unit("psi")
            .arc(12.0, 16.0, IndicatorStatus::Nominal)
            .arc(16.0, 18.0, IndicatorStatus::Caution)
            .arc(18.0, 20.0, IndicatorStatus::Alert)
            .major_ticks(5)
            .minor_ticks(4);
    }

    #[test]
    fn gauge_needle_color_nominal_is_white() {
        let gauge = Gauge::new("test", 50.0, 0.0, 100.0)
            .arc(0.0, 100.0, IndicatorStatus::Nominal);
        let angle = gauge.value_to_angle(50.0);
        let color = gauge.needle_color_for_angle(angle);
        assert_eq!(color, gauge.colors.needle);
    }

    #[test]
    fn gauge_needle_color_in_danger_zone_is_tinted() {
        let gauge = Gauge::new("test", 90.0, 0.0, 100.0)
            .arc(80.0, 100.0, IndicatorStatus::Alert);
        let angle = gauge.value_to_angle(90.0);
        let color = gauge.needle_color_for_angle(angle);
        // Should be a blend of white and alert red — not pure white
        assert!(color.r > 0.8, "Should have strong red component");
        assert!(color != gauge.colors.needle, "Should differ from pure white needle");
    }
}
