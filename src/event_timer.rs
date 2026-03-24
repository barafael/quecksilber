use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Event, Frame, Geometry, Text};
use iced::{Point, Rectangle, Renderer, Theme};

use std::time::Duration;

use crate::anim::Spring;
use crate::color::MercuryColors;
use crate::draw;

/// Timer display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, strum::Display, strum::EnumString)]
pub enum TimerMode {
    /// Counts upward from zero (mission elapsed time).
    #[default]
    CountUp,
    /// Counts downward toward zero (countdown).
    CountDown,
}

/// An analog clock-face elapsed time / countdown display.
///
/// Faithful to the Mercury instrument panel's mechanical timer: a round dial
/// with 60-second markings, a sweep hand for seconds, and numeric labels for
/// minutes and hours at the center. Uses the same gauge rendering foundations
/// (bezel, ticks, needle).
///
/// The timer value is driven externally — the consuming app updates the `elapsed`
/// field via `iced::time::every`.
#[derive(Debug, Clone)]
pub struct EventTimer {
    elapsed: Duration,
    mode: TimerMode,
    label: String,
    running: bool,
    colors: MercuryColors,
}

impl EventTimer {
    /// Creates a new event timer.
    #[must_use]
    pub fn new(label: impl Into<String>, elapsed: Duration) -> Self {
        Self {
            elapsed,
            mode: TimerMode::CountUp,
            label: label.into(),
            running: false,
            colors: MercuryColors::default(),
        }
    }

    /// Sets the timer mode (count up or count down).
    #[must_use]
    pub fn mode(mut self, mode: TimerMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets whether the timer is currently running (affects sweep hand animation).
    #[must_use]
    pub fn running(mut self, running: bool) -> Self {
        self.running = running;
        self
    }

    /// Sets a custom color palette.
    #[must_use]
    pub fn colors(mut self, colors: MercuryColors) -> Self {
        self.colors = colors;
        self
    }

    fn hours(&self) -> u64 {
        self.elapsed.as_secs() / 3600
    }

    fn minutes(&self) -> u64 {
        (self.elapsed.as_secs() % 3600) / 60
    }

    fn seconds(&self) -> u64 {
        self.elapsed.as_secs() % 60
    }

    fn seconds_fraction(&self) -> f32 {
        let secs = self.elapsed.as_secs() % 60;
        let millis = self.elapsed.subsec_millis();
        secs as f32 + millis as f32 / 1000.0
    }
}

/// Internal state for the event timer's animation and cache.
#[derive(Debug)]
pub struct EventTimerState {
    cache: Cache<Renderer>,
    sweep_spring: Spring,
}

impl Default for EventTimerState {
    fn default() -> Self {
        Self {
            cache: Cache::default(),
            // Critically damped for smooth second-hand sweep
            sweep_spring: Spring::with_params(0.0, 200.0, 1.0),
        }
    }
}

impl canvas::Program<(), Theme, Renderer> for EventTimer {
    type State = EventTimerState;

    fn update(
        &self,
        state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<()>> {
        // Map seconds to angle (0 seconds = 12 o'clock = -π/2)
        let target_angle =
            -std::f32::consts::FRAC_PI_2 + self.seconds_fraction() * std::f32::consts::TAU / 60.0;
        state.sweep_spring.set_target(target_angle);

        if !state.sweep_spring.is_settled() {
            state.sweep_spring.tick(1.0 / 60.0);
            Some(canvas::Action::request_redraw())
        } else if self.running {
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

        // Static geometry (cached)
        let static_geo = state.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_static(frame, center, radius);
        });

        // Dynamic sweep hand
        let mut hand_frame = Frame::new(renderer, bounds.size());
        let sweep_angle = state.sweep_spring.value();
        draw::draw_needle(&mut hand_frame, center, radius * 0.80, sweep_angle, self.colors.needle);

        // Hub dot
        let hub = canvas::Path::circle(center, 3.0);
        hand_frame.fill(&hub, self.colors.needle);

        vec![static_geo, hand_frame.into_geometry()]
    }
}

impl EventTimer {
    fn draw_static(&self, frame: &mut Frame<Renderer>, center: Point, radius: f32) {
        let tick_outer = radius * 0.90;
        let tick_inner_major = radius * 0.78;
        let tick_inner_minor = radius * 0.84;

        // Background circle
        let bg = canvas::Path::circle(center, radius);
        frame.fill(&bg, self.colors.panel_bg);

        // Bezel
        draw::draw_bezel(frame, center, radius, self.colors.bezel, 2.0);

        // 60-second tick marks
        for i in 0..60 {
            let angle =
                -std::f32::consts::FRAC_PI_2 + (i as f32 / 60.0) * std::f32::consts::TAU;

            if i % 5 == 0 {
                // Major tick every 5 seconds
                draw::draw_tick(
                    frame,
                    center,
                    tick_inner_major,
                    tick_outer,
                    angle,
                    self.colors.tick_mark,
                    1.5,
                );

                // Numeric label (seconds: 0, 5, 10, ..., 55)
                let label_radius = radius * 0.65;
                let (sin, cos) = angle.sin_cos();
                let label_pos = Point::new(
                    center.x + cos * label_radius,
                    center.y + sin * label_radius,
                );
                frame.fill_text(Text {
                    content: format!("{i}"),
                    position: label_pos,
                    color: self.colors.tick_mark,
                    size: iced::Pixels(9.0),
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center,
                    font: self.colors.font,
            ..Text::default()
                });
            } else {
                // Minor tick
                draw::draw_tick(
                    frame,
                    center,
                    tick_inner_minor,
                    tick_outer,
                    angle,
                    self.colors.tick_mark,
                    0.5,
                );
            }
        }

        // All text below center to stay clear of the sweep hand.

        // Label
        frame.fill_text(Text {
            content: self.label.clone(),
            position: Point::new(center.x, center.y + radius * 0.10),
            color: self.colors.primary,
            size: iced::Pixels(10.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            font: self.colors.font,
            ..Text::default()
        });

        // Digital time readout
        let time_str = format!(
            "{:02}:{:02}:{:02}",
            self.hours(),
            self.minutes(),
            self.seconds()
        );
        frame.fill_text(Text {
            content: time_str,
            position: Point::new(center.x, center.y + radius * 0.30),
            color: self.colors.text,
            size: iced::Pixels(12.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            font: self.colors.font,
            ..Text::default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn timer_mode_roundtrip() {
        for mode in [TimerMode::CountUp, TimerMode::CountDown] {
            let s = mode.to_string();
            let parsed = TimerMode::from_str(&s).unwrap();
            assert_eq!(parsed, mode);
        }
    }

    #[test]
    fn timer_mode_from_str_invalid() {
        assert!(TimerMode::from_str("garbage").is_err());
    }

    #[test]
    fn timer_hms_decomposition() {
        let timer = EventTimer::new("T", Duration::from_secs(3661)); // 1h 1m 1s
        assert_eq!(timer.hours(), 1);
        assert_eq!(timer.minutes(), 1);
        assert_eq!(timer.seconds(), 1);
    }

    #[test]
    fn timer_seconds_fraction() {
        let timer = EventTimer::new("T", Duration::from_millis(30500)); // 30.5s
        let frac = timer.seconds_fraction();
        assert!((frac - 30.5).abs() < 0.01);
    }

    #[test]
    fn timer_zero() {
        let timer = EventTimer::new("T", Duration::ZERO);
        assert_eq!(timer.hours(), 0);
        assert_eq!(timer.minutes(), 0);
        assert_eq!(timer.seconds(), 0);
    }
}
