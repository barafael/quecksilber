use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{Color, Point, Renderer, Size};

use std::f32::consts::PI;

/// Draws a circular bezel (outline) on the frame.
pub fn draw_bezel(
    frame: &mut Frame<Renderer>,
    center: Point,
    radius: f32,
    color: Color,
    width: f32,
) {
    let circle = Path::circle(center, radius);
    frame.stroke(&circle, Stroke::default().with_color(color).with_width(width));
}

/// Draws a radial tick mark (a line segment from inner radius to outer radius at a given angle).
pub fn draw_tick(
    frame: &mut Frame<Renderer>,
    center: Point,
    radius_inner: f32,
    radius_outer: f32,
    angle: f32,
    color: Color,
    width: f32,
) {
    let (sin, cos) = angle.sin_cos();
    let start = Point::new(center.x + cos * radius_inner, center.y + sin * radius_inner);
    let end = Point::new(center.x + cos * radius_outer, center.y + sin * radius_outer);

    let path = Path::line(start, end);
    frame.stroke(&path, Stroke::default().with_color(color).with_width(width));
}

/// Draws a tapered needle from near the center to the arc radius at a given angle.
pub fn draw_needle(
    frame: &mut Frame<Renderer>,
    center: Point,
    radius: f32,
    angle: f32,
    color: Color,
) {
    let (sin, cos) = angle.sin_cos();
    let tip = Point::new(center.x + cos * radius, center.y + sin * radius);

    // The needle is a thin triangle: narrow base at center, point at tip.
    let base_half_width = 3.0;
    let perpendicular_angle = angle + PI / 2.0;
    let (psin, pcos) = perpendicular_angle.sin_cos();

    let base_left = Point::new(
        center.x + pcos * base_half_width,
        center.y + psin * base_half_width,
    );
    let base_right = Point::new(
        center.x - pcos * base_half_width,
        center.y - psin * base_half_width,
    );

    let needle = Path::new(|builder| {
        builder.move_to(base_left);
        builder.line_to(tip);
        builder.line_to(base_right);
        builder.close();
    });

    frame.fill(&needle, color);
}

/// Draws an arc segment (a thick arc between two angles) for gauge zones.
pub fn draw_arc_segment(
    frame: &mut Frame<Renderer>,
    center: Point,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    width: f32,
    color: Color,
) {
    let arc = Path::new(|builder| {
        builder.arc(iced::widget::canvas::path::Arc {
            center,
            radius,
            start_angle: iced::Radians(start_angle),
            end_angle: iced::Radians(end_angle),
        });
    });

    frame.stroke(&arc, Stroke::default().with_color(color).with_width(width));
}

/// Draws a soft rectangular glow behind an indicator light.
pub fn draw_glow(
    frame: &mut Frame<Renderer>,
    position: Point,
    size: Size,
    color: Color,
    spread: f32,
) {
    let expanded = Path::rectangle(
        Point::new(position.x - spread, position.y - spread),
        Size::new(size.width + spread * 2.0, size.height + spread * 2.0),
    );
    let glow_color = Color { a: 0.3 * color.a, ..color };
    frame.fill(&expanded, glow_color);
}

/// Converts a value within a range to an angle on the gauge sweep.
///
/// `start_angle` is where the minimum value maps (e.g. 225 degrees = 7 o'clock).
/// `sweep_angle` is the total angular range (e.g. 270 degrees).
/// All angles in radians.
pub fn value_to_angle(
    value: f32,
    range_start: f32,
    range_end: f32,
    start_angle: f32,
    sweep_angle: f32,
) -> f32 {
    let range = range_end - range_start;
    if range.abs() < f32::EPSILON {
        return start_angle;
    }
    let normalized = ((value - range_start) / range).clamp(0.0, 1.0);
    start_angle + normalized * sweep_angle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_to_angle_at_start() {
        let angle = value_to_angle(0.0, 0.0, 100.0, 0.0, PI);
        assert!((angle - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn value_to_angle_at_end() {
        let angle = value_to_angle(100.0, 0.0, 100.0, 0.0, PI);
        assert!((angle - PI).abs() < f32::EPSILON);
    }

    #[test]
    fn value_to_angle_at_midpoint() {
        let angle = value_to_angle(50.0, 0.0, 100.0, 0.0, PI);
        assert!((angle - PI / 2.0).abs() < 0.001);
    }

    #[test]
    fn value_to_angle_with_offset_start() {
        let start = 3.0 * PI / 4.0; // 135 degrees
        let sweep = 3.0 * PI / 2.0; // 270 degrees
        let angle = value_to_angle(0.0, 0.0, 100.0, start, sweep);
        assert!((angle - start).abs() < f32::EPSILON);
    }

    #[test]
    fn value_to_angle_clamps_below_range() {
        let angle = value_to_angle(-10.0, 0.0, 100.0, 0.0, PI);
        assert!((angle - 0.0).abs() < f32::EPSILON, "Below range should clamp to start");
    }

    #[test]
    fn value_to_angle_clamps_above_range() {
        let angle = value_to_angle(200.0, 0.0, 100.0, 0.0, PI);
        assert!((angle - PI).abs() < f32::EPSILON, "Above range should clamp to end");
    }

    #[test]
    fn value_to_angle_zero_range_returns_start() {
        let angle = value_to_angle(50.0, 100.0, 100.0, 1.0, PI);
        assert!((angle - 1.0).abs() < f32::EPSILON, "Zero range should return start_angle");
    }
}
