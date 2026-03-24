use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Cache, Event, Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Point, Rectangle, Renderer, Theme};

use crate::anim::Spring;
use crate::color::MercuryColors;
use crate::draw;

/// A three-axis attitude direction indicator showing roll, pitch, and yaw
/// on independent linear scales.
///
/// Inspired by the Mercury spacecraft ADI — a circular instrument with three
/// scales arranged in a rectangular frame around a central spacecraft
/// silhouette. Each axis has an animated triangle pointer driven by spring
/// physics.
///
/// Roll is displayed along the top edge, pitch along the right edge, and
/// yaw along the bottom edge. The pointers slide along their respective
/// scales and animate smoothly when values change.
#[derive(Debug, Clone)]
pub struct AttitudeIndicator {
    roll: f32,
    pitch: f32,
    yaw: f32,
    roll_range: (f32, f32),
    pitch_range: (f32, f32),
    yaw_range: (f32, f32),
    colors: MercuryColors,
}

impl AttitudeIndicator {
    /// Creates a new attitude indicator with the given roll, pitch, and yaw values.
    ///
    /// Default ranges are +-180 for roll and yaw, +-90 for pitch.
    #[must_use]
    pub fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self {
            roll,
            pitch,
            yaw,
            roll_range: (-180.0, 180.0),
            pitch_range: (-90.0, 90.0),
            yaw_range: (-180.0, 180.0),
            colors: MercuryColors::default(),
        }
    }

    /// Sets a custom roll range (min, max) in degrees.
    #[must_use]
    pub fn roll_range(mut self, min: f32, max: f32) -> Self {
        self.roll_range = (min, max);
        self
    }

    /// Sets a custom pitch range (min, max) in degrees.
    #[must_use]
    pub fn pitch_range(mut self, min: f32, max: f32) -> Self {
        self.pitch_range = (min, max);
        self
    }

    /// Sets a custom yaw range (min, max) in degrees.
    #[must_use]
    pub fn yaw_range(mut self, min: f32, max: f32) -> Self {
        self.yaw_range = (min, max);
        self
    }

    /// Sets a custom color palette.
    #[must_use]
    pub fn colors(mut self, colors: MercuryColors) -> Self {
        self.colors = colors;
        self
    }

    /// Maps a value within [min, max] to [0.0, 1.0], clamped.
    fn normalize(value: f32, min: f32, max: f32) -> f32 {
        let range = max - min;
        if range.abs() < f32::EPSILON {
            return 0.5;
        }
        ((value - min) / range).clamp(0.0, 1.0)
    }

    /// Returns the inner rectangle bounds (left, top, right, bottom).
    fn rect_bounds(center: Point, radius: f32) -> (f32, f32, f32, f32) {
        let half = radius * 0.64;
        (
            center.x - half,
            center.y - half,
            center.x + half,
            center.y + half,
        )
    }
}

/// Internal state for the attitude indicator's pointer animations and geometry cache.
#[derive(Debug)]
pub struct AttitudeIndicatorState {
    cache: Cache<Renderer>,
    roll_spring: Spring,
    pitch_spring: Spring,
    yaw_spring: Spring,
}

impl Default for AttitudeIndicatorState {
    fn default() -> Self {
        Self {
            cache: Cache::default(),
            roll_spring: Spring::with_params(0.5, 180.0, 0.85),
            pitch_spring: Spring::with_params(0.5, 180.0, 0.85),
            yaw_spring: Spring::with_params(0.5, 180.0, 0.85),
        }
    }
}

impl canvas::Program<(), Theme, Renderer> for AttitudeIndicator {
    type State = AttitudeIndicatorState;

    fn update(
        &self,
        state: &mut Self::State,
        _event: &Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<()>> {
        let roll_target = Self::normalize(self.roll, self.roll_range.0, self.roll_range.1);
        let pitch_target = Self::normalize(self.pitch, self.pitch_range.0, self.pitch_range.1);
        let yaw_target = Self::normalize(self.yaw, self.yaw_range.0, self.yaw_range.1);

        state.roll_spring.set_target(roll_target);
        state.pitch_spring.set_target(pitch_target);
        state.yaw_spring.set_target(yaw_target);

        let all_settled = state.roll_spring.is_settled()
            && state.pitch_spring.is_settled()
            && state.yaw_spring.is_settled();

        if !all_settled {
            let dt = 1.0 / 60.0;
            state.roll_spring.tick(dt);
            state.pitch_spring.tick(dt);
            state.yaw_spring.tick(dt);
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

        // Dynamic geometry (pointers — redrawn each frame during animation)
        let mut pointer_frame = Frame::new(renderer, bounds.size());
        self.draw_pointers(
            &mut pointer_frame,
            center,
            radius,
            state.roll_spring.value(),
            state.pitch_spring.value(),
            state.yaw_spring.value(),
        );

        vec![static_geo, pointer_frame.into_geometry()]
    }
}

// ── Drawing helpers ──────────────────────────────────────────────────

impl AttitudeIndicator {
    fn draw_static(&self, frame: &mut Frame<Renderer>, center: Point, radius: f32) {
        let (left, top, right, bottom) = Self::rect_bounds(center, radius);
        let rect_half = radius * 0.64;

        // Background circle
        let bg = Path::circle(center, radius);
        frame.fill(&bg, self.colors.panel_bg);

        // Bezel
        draw::draw_bezel(frame, center, radius, self.colors.bezel, 2.0);

        // Inner rectangular frame
        let rect_path = Path::new(|b| {
            b.move_to(Point::new(left, top));
            b.line_to(Point::new(right, top));
            b.line_to(Point::new(right, bottom));
            b.line_to(Point::new(left, bottom));
            b.close();
        });
        frame.stroke(
            &rect_path,
            Stroke::default()
                .with_color(self.colors.needle)
                .with_width(1.0),
        );

        // Tick marks on three edges
        draw_horizontal_ticks(frame, left, right, top, true, &self.colors);
        draw_vertical_ticks(frame, top, bottom, right, true, &self.colors);
        draw_horizontal_ticks(frame, left, right, bottom, false, &self.colors);

        // Labels
        let label_gap = (radius - rect_half) * 0.72;

        // ROLL — above top edge, centered
        frame.fill_text(Text {
            content: "ROLL".to_string(),
            position: Point::new(center.x, top - label_gap),
            color: self.colors.text,
            size: iced::Pixels(10.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            ..Text::default()
        });

        // PITCH — right of right edge, stacked vertically
        let pitch_chars = ['P', 'I', 'T', 'C', 'H'];
        let char_spacing = 11.0;
        let pitch_x = right + label_gap;
        let pitch_start_y = center.y - (pitch_chars.len() as f32 * char_spacing) / 2.0
            + char_spacing / 2.0;
        for (i, ch) in pitch_chars.iter().enumerate() {
            frame.fill_text(Text {
                content: ch.to_string(),
                position: Point::new(pitch_x, pitch_start_y + i as f32 * char_spacing),
                color: self.colors.text,
                size: iced::Pixels(10.0),
                align_x: iced::alignment::Horizontal::Center.into(),
                align_y: iced::alignment::Vertical::Center,
                ..Text::default()
            });
        }

        // YAW — below bottom edge, centered
        frame.fill_text(Text {
            content: "YAW".to_string(),
            position: Point::new(center.x, bottom + label_gap),
            color: self.colors.text,
            size: iced::Pixels(10.0),
            align_x: iced::alignment::Horizontal::Center.into(),
            align_y: iced::alignment::Vertical::Center,
            ..Text::default()
        });

        // Spacecraft silhouette
        draw_spacecraft(frame, center, rect_half, self.colors.needle);
    }

    fn draw_pointers(
        &self,
        frame: &mut Frame<Renderer>,
        center: Point,
        radius: f32,
        roll_frac: f32,
        pitch_frac: f32,
        yaw_frac: f32,
    ) {
        let (left, top, right, bottom) = Self::rect_bounds(center, radius);
        let width = right - left;
        let height = bottom - top;
        let size = 7.0;

        // Roll — triangle above top edge, pointing down
        let roll_x = left + roll_frac * width;
        draw_triangle(frame, Point::new(roll_x, top), size, Direction::Down, self.colors.needle);

        // Pitch — triangle right of right edge, pointing left
        let pitch_y = top + pitch_frac * height;
        draw_triangle(frame, Point::new(right, pitch_y), size, Direction::Left, self.colors.needle);

        // Yaw — triangle below bottom edge, pointing up
        let yaw_x = left + yaw_frac * width;
        draw_triangle(frame, Point::new(yaw_x, bottom), size, Direction::Up, self.colors.needle);
    }
}

// ── Free drawing functions ───────────────────────────────────────────

/// Direction a triangle pointer faces (toward the center of the rectangle).
enum Direction {
    Up,
    Down,
    Left,
}

/// Draws a filled triangle pointer at `tip` with the apex at the rectangle edge
/// and the body extending outward.
fn draw_triangle(
    frame: &mut Frame<Renderer>,
    tip: Point,
    size: f32,
    direction: Direction,
    color: Color,
) {
    let half_base = size * 0.6;
    let (p1, p2) = match direction {
        Direction::Down => (
            Point::new(tip.x - half_base, tip.y - size),
            Point::new(tip.x + half_base, tip.y - size),
        ),
        Direction::Up => (
            Point::new(tip.x - half_base, tip.y + size),
            Point::new(tip.x + half_base, tip.y + size),
        ),
        Direction::Left => (
            Point::new(tip.x + size, tip.y - half_base),
            Point::new(tip.x + size, tip.y + half_base),
        ),
    };

    let triangle = Path::new(|b| {
        b.move_to(tip);
        b.line_to(p1);
        b.line_to(p2);
        b.close();
    });
    frame.fill(&triangle, color);
}

/// Draws tick marks along a horizontal edge (top or bottom of the rectangle frame).
fn draw_horizontal_ticks(
    frame: &mut Frame<Renderer>,
    left: f32,
    right: f32,
    y: f32,
    inward_is_down: bool,
    colors: &MercuryColors,
) {
    let width = right - left;
    let major_count: u32 = 10;
    let minor_per_major: u32 = 4;
    let total = major_count * minor_per_major;
    let major_len = 8.0;
    let minor_len = 4.0;
    let dir = if inward_is_down { 1.0 } else { -1.0 };

    for i in 0..=total {
        let frac = i as f32 / total as f32;
        let x = left + frac * width;
        let is_major = i % minor_per_major == 0;
        let len = if is_major { major_len } else { minor_len };
        let w = if is_major { 1.5 } else { 0.75 };

        let path = Path::line(Point::new(x, y), Point::new(x, y + dir * len));
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(colors.tick_mark)
                .with_width(w),
        );
    }
}

/// Draws tick marks along the right vertical edge of the rectangle frame.
fn draw_vertical_ticks(
    frame: &mut Frame<Renderer>,
    top: f32,
    bottom: f32,
    x: f32,
    inward_is_left: bool,
    colors: &MercuryColors,
) {
    let height = bottom - top;
    let major_count: u32 = 6;
    let minor_per_major: u32 = 4;
    let total = major_count * minor_per_major;
    let major_len = 8.0;
    let minor_len = 4.0;
    let dir = if inward_is_left { -1.0 } else { 1.0 };

    for i in 0..=total {
        let frac = i as f32 / total as f32;
        let y = top + frac * height;
        let is_major = i % minor_per_major == 0;
        let len = if is_major { major_len } else { minor_len };
        let w = if is_major { 1.5 } else { 0.75 };

        let path = Path::line(Point::new(x, y), Point::new(x + dir * len, y));
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(colors.tick_mark)
                .with_width(w),
        );
    }
}

/// Draws a stylized plan-view spacecraft silhouette at center.
fn draw_spacecraft(
    frame: &mut Frame<Renderer>,
    center: Point,
    rect_half: f32,
    color: Color,
) {
    let thin = Stroke::default().with_color(color).with_width(1.5);
    let thick = Stroke::default().with_color(color).with_width(2.5);

    // Fuselage — vertical line from nose to tail
    let fuse_top = center.y - rect_half * 0.65;
    let fuse_bot = center.y + rect_half * 0.52;
    frame.stroke(
        &Path::line(
            Point::new(center.x, fuse_top),
            Point::new(center.x, fuse_bot),
        ),
        thick,
    );

    // Wings — wide horizontal line slightly above center
    let wing_y = center.y - rect_half * 0.08;
    let wing_half = rect_half * 0.72;
    frame.stroke(
        &Path::line(
            Point::new(center.x - wing_half, wing_y),
            Point::new(center.x + wing_half, wing_y),
        ),
        thin,
    );

    // Body section at wing junction — small filled rectangle
    let body_hw = rect_half * 0.08;
    let body_hh = rect_half * 0.12;
    let body = Path::new(|b| {
        b.move_to(Point::new(center.x - body_hw, wing_y - body_hh));
        b.line_to(Point::new(center.x + body_hw, wing_y - body_hh));
        b.line_to(Point::new(center.x + body_hw, wing_y + body_hh));
        b.line_to(Point::new(center.x - body_hw, wing_y + body_hh));
        b.close();
    });
    frame.fill(&body, color);

    // Horizontal stabilizer — shorter horizontal line below center
    let tail_y = center.y + rect_half * 0.35;
    let tail_half = rect_half * 0.28;
    frame.stroke(
        &Path::line(
            Point::new(center.x - tail_half, tail_y),
            Point::new(center.x + tail_half, tail_y),
        ),
        thin,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_at_boundaries() {
        assert!((AttitudeIndicator::normalize(0.0, 0.0, 100.0) - 0.0).abs() < f32::EPSILON);
        assert!((AttitudeIndicator::normalize(100.0, 0.0, 100.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_at_midpoint() {
        let mid = AttitudeIndicator::normalize(50.0, 0.0, 100.0);
        assert!((mid - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_clamps_below_range() {
        let val = AttitudeIndicator::normalize(-10.0, 0.0, 100.0);
        assert!((val - 0.0).abs() < f32::EPSILON, "Below range should clamp to 0.0");
    }

    #[test]
    fn normalize_clamps_above_range() {
        let val = AttitudeIndicator::normalize(200.0, 0.0, 100.0);
        assert!((val - 1.0).abs() < f32::EPSILON, "Above range should clamp to 1.0");
    }

    #[test]
    fn normalize_zero_range_returns_center() {
        let val = AttitudeIndicator::normalize(50.0, 100.0, 100.0);
        assert!(
            (val - 0.5).abs() < f32::EPSILON,
            "Zero range should return 0.5"
        );
    }

    #[test]
    fn normalize_negative_range() {
        let val = AttitudeIndicator::normalize(0.0, -180.0, 180.0);
        assert!((val - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn construction_does_not_panic() {
        let _ = AttitudeIndicator::new(5.0, -2.0, 1.5)
            .roll_range(-30.0, 30.0)
            .pitch_range(-20.0, 20.0)
            .yaw_range(-30.0, 30.0)
            .colors(MercuryColors::default());
    }

    #[test]
    fn default_ranges() {
        let adi = AttitudeIndicator::new(0.0, 0.0, 0.0);
        assert_eq!(adi.roll_range, (-180.0, 180.0));
        assert_eq!(adi.pitch_range, (-90.0, 90.0));
        assert_eq!(adi.yaw_range, (-180.0, 180.0));
    }

    #[test]
    fn rect_bounds_are_symmetric() {
        let center = Point::new(100.0, 100.0);
        let radius = 80.0;
        let (left, top, right, bottom) = AttitudeIndicator::rect_bounds(center, radius);
        assert!((center.x - left - (right - center.x)).abs() < f32::EPSILON);
        assert!((center.y - top - (bottom - center.y)).abs() < f32::EPSILON);
    }
}
