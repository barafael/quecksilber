use iced::widget::canvas::{self, Path, Stroke};
use iced::{Element, Font, Length, Point, Radians, Rectangle, Renderer, Theme, mouse};

/// A dual-gauge widget displaying two values on a single circular face.
pub struct DualGauge {
    top_label: String,
    right_label: String,
    bottom_label: String,
    left_label: String,
    font: Font,
    left_min: f32,
    left_max: f32,
    left_label_every: u32,
    left_value: f32,
    right_min: f32,
    right_max: f32,
    right_label_every: u32,
    right_value: f32,
}

impl DualGauge {
    pub fn new() -> Self {
        Self {
            top_label: String::new(),
            right_label: String::new(),
            bottom_label: String::new(),
            left_label: String::new(),
            font: Font::default(),
            left_min: 0.0,
            left_max: 100.0,
            left_label_every: 10,
            left_value: 0.0,
            right_min: 0.0,
            right_max: 100.0,
            right_label_every: 10,
            right_value: 0.0,
        }
    }

    pub fn top_label(mut self, label: impl Into<String>) -> Self {
        self.top_label = label.into();
        self
    }

    pub fn right_label(mut self, label: impl Into<String>) -> Self {
        self.right_label = label.into();
        self
    }

    pub fn bottom_label(mut self, label: impl Into<String>) -> Self {
        self.bottom_label = label.into();
        self
    }

    pub fn left_label(mut self, label: impl Into<String>) -> Self {
        self.left_label = label.into();
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn left_range(mut self, min: f32, max: f32) -> Self {
        self.left_min = min;
        self.left_max = max;
        self
    }

    pub fn right_range(mut self, min: f32, max: f32) -> Self {
        self.right_min = min;
        self.right_max = max;
        self
    }

    pub fn left_label_every(mut self, n: u32) -> Self {
        self.left_label_every = n;
        self
    }

    pub fn right_label_every(mut self, n: u32) -> Self {
        self.right_label_every = n;
        self
    }

    pub fn left_value(mut self, value: f32) -> Self {
        self.left_value = value;
        self
    }

    pub fn right_value(mut self, value: f32) -> Self {
        self.right_value = value;
        self
    }

    pub fn set_left_value(&mut self, value: f32) {
        self.left_value = value;
    }

    pub fn set_right_value(&mut self, value: f32) {
        self.right_value = value;
    }

    pub fn view<Message: 'static>(&self) -> Element<'_, Message> {
        iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_arm(
        &self,
        frame: &mut canvas::Frame,
        fg: iced::Color,
        arc_center: Point,
        arc_radius: f32,
        inner_radius: f32,
        start_angle: f32,
        end_angle: f32,
        min: f32,
        max: f32,
        value: f32,
        full_radius: f32,
    ) {
        let range_span = max - min;
        let t = ((value - min) / range_span).clamp(0.0, 1.0);
        let sweep = end_angle - start_angle;
        let angle = start_angle + t * sweep;

        let cos = angle.cos();
        let sin = angle.sin();
        let perp_cos = -sin;
        let perp_sin = cos;

        let half_width = full_radius * 0.05;
        let arm_start = inner_radius;
        let arm_end = arc_radius * 0.6;
        let tip_length = full_radius * 0.18;

        let arm = Path::new(|b| {
            let base_l = Point::new(
                arc_center.x + cos * arm_start + perp_cos * half_width,
                arc_center.y + sin * arm_start + perp_sin * half_width,
            );
            let base_r = Point::new(
                arc_center.x + cos * arm_start - perp_cos * half_width,
                arc_center.y + sin * arm_start - perp_sin * half_width,
            );
            let top_l = Point::new(
                arc_center.x + cos * arm_end + perp_cos * half_width,
                arc_center.y + sin * arm_end + perp_sin * half_width,
            );
            let top_r = Point::new(
                arc_center.x + cos * arm_end - perp_cos * half_width,
                arc_center.y + sin * arm_end - perp_sin * half_width,
            );
            let tip = Point::new(
                arc_center.x + cos * (arm_end + tip_length),
                arc_center.y + sin * (arm_end + tip_length),
            );

            b.move_to(base_l);
            b.line_to(top_l);
            b.line_to(tip);
            b.line_to(top_r);
            b.line_to(base_r);
            b.close();
        });
        frame.fill(&arm, fg);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_arc_ticks(
        &self,
        frame: &mut canvas::Frame,
        fg: iced::Color,
        arc_center: Point,
        arc_radius: f32,
        start_angle: f32,
        end_angle: f32,
        min: f32,
        max: f32,
        label_every: u32,
        full_radius: f32,
    ) {
        let range_span = max - min;
        let sweep = end_angle - start_angle;

        let tick_len_label = arc_radius * 0.12;
        let tick_len_small = arc_radius * 0.06;
        let tick_w_label = (full_radius * 0.02).max(1.0);
        let tick_w_small = (full_radius * 0.01).max(0.5);
        let label_offset = arc_radius * 0.2;
        let label_size = full_radius * 0.10;

        // Small ticks at 1/5 of label_every intervals
        let label_step = label_every.max(1) as f32;
        let tick_step = label_step / 5.0;
        let total_ticks = (range_span / tick_step).round() as u32;

        for i in 0..=total_ticks {
            let val = min + i as f32 * tick_step;
            if val > max + 0.01 {
                break;
            }
            let val = val.min(max);

            let t = (val - min) / range_span;
            let angle = start_angle + t * sweep;
            let cos = angle.cos();
            let sin = angle.sin();

            let val_from_min = val - min;
            let is_label_tick = (val_from_min % label_step).abs() < 0.01
                || (val_from_min % label_step - label_step).abs() < 0.01;
            let is_endpoint = (val - min).abs() < 0.01 || (val - max).abs() < 0.01;
            let show_label = is_label_tick || is_endpoint;

            let tick_len = if show_label {
                tick_len_label
            } else {
                tick_len_small
            };
            let tick_w = if show_label {
                tick_w_label
            } else {
                tick_w_small
            };

            let outer = Point::new(
                arc_center.x + cos * arc_radius,
                arc_center.y + sin * arc_radius,
            );
            let inner = Point::new(
                arc_center.x + cos * (arc_radius - tick_len),
                arc_center.y + sin * (arc_radius - tick_len),
            );

            let tick = Path::line(inner, outer);
            frame.stroke(&tick, Stroke::default().with_color(fg).with_width(tick_w));

            if show_label {
                let label_pos = Point::new(
                    arc_center.x + cos * (arc_radius - tick_len - label_offset),
                    arc_center.y + sin * (arc_radius - tick_len - label_offset),
                );
                frame.fill_text(crate::centered_text(
                    format!("{}", val as i32),
                    label_pos,
                    label_size,
                    fg,
                    self.font,
                ));
            }
        }
    }

    /// Draw the dual gauge at a specific center point and radius on the given frame.
    pub fn draw_at(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
    ) {
        let primary = theme.palette().primary;
        let fg = theme.palette().background;
        let font_size = full_radius * 0.12;

        let bg_circle = Path::circle(center, full_radius);
        frame.fill(&bg_circle, primary);

        let small_r = full_radius / 4.0;

        // Vertical line: x at 1/5 off the rightmost point of the big circle
        let line_x = center.x + full_radius * 0.64;
        let dx_big = line_x - center.x;
        let half_chord = crate::half_chord(full_radius, dx_big);
        let y_top = center.y - half_chord;
        let y_bottom = center.y + half_chord;

        let stroke = canvas::Stroke::default()
            .with_color(fg)
            .with_width((full_radius * 0.015).max(1.0));

        // Where the vertical line meets the rightmost small circle
        let right_small_cx = center.x + full_radius - small_r;
        let right_small_cy = center.y;
        let dx_small = line_x - right_small_cx;
        let arc_r = small_r * 1.16;
        let small_half_chord = crate::half_chord(arc_r, dx_small);
        let y_small_top = right_small_cy - small_half_chord;
        let y_small_bottom = right_small_cy + small_half_chord;

        // Vertical line split: above and below the small circle
        let line_top = Path::line(Point::new(line_x, y_top), Point::new(line_x, y_small_top));
        frame.stroke(&line_top, stroke);
        let line_bottom = Path::line(
            Point::new(line_x, y_small_bottom),
            Point::new(line_x, y_bottom),
        );
        frame.stroke(&line_bottom, stroke);

        // Arc: left half of the rightmost small circle area, slightly larger
        let arc_intersect = (dx_small / arc_r).clamp(-1.0, 1.0).acos();
        let arc_path = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: Point::new(right_small_cx, right_small_cy),
                radius: arc_r,
                start_angle: Radians(arc_intersect),
                end_angle: Radians(-arc_intersect + 2.0 * std::f32::consts::PI),
            });
        });
        frame.stroke(&arc_path, stroke);

        // Larger gauge arc on the right side (same center, 3x radius)
        let gauge_arc_r = arc_r * 2.58;
        let gauge_margin = 0.38;
        let gauge_dx = dx_small / gauge_arc_r;
        let gauge_intersect = gauge_dx.clamp(-1.0, 1.0).acos();
        let gauge_arc_right = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: Point::new(right_small_cx, right_small_cy),
                radius: gauge_arc_r,
                start_angle: Radians(gauge_intersect + gauge_margin),
                end_angle: Radians(-gauge_intersect - gauge_margin + 2.0 * std::f32::consts::PI),
            });
        });
        frame.stroke(&gauge_arc_right, stroke);

        // Ticks on the right gauge arc (0 at bottom, values increase upward)
        self.draw_arc_ticks(
            frame,
            fg,
            Point::new(right_small_cx, right_small_cy),
            gauge_arc_r,
            gauge_intersect + gauge_margin,
            -gauge_intersect - gauge_margin + 2.0 * std::f32::consts::PI,
            self.right_min,
            self.right_max,
            self.right_label_every,
            full_radius,
        );

        // Arm on the right gauge
        self.draw_arm(
            frame,
            fg,
            Point::new(right_small_cx, right_small_cy),
            gauge_arc_r,
            arc_r,
            gauge_intersect + gauge_margin,
            -gauge_intersect - gauge_margin + 2.0 * std::f32::consts::PI,
            self.right_min,
            self.right_max,
            self.right_value,
            full_radius,
        );

        // Mirror on the left side
        let line_x_left = center.x - full_radius * 0.64;
        let dx_big_left = line_x_left - center.x;
        let half_chord_left = crate::half_chord(full_radius, dx_big_left);
        let y_top_left = center.y - half_chord_left;
        let y_bottom_left = center.y + half_chord_left;

        let left_small_cx = center.x - full_radius + small_r;
        let left_small_cy = center.y;
        let dx_small_left = line_x_left - left_small_cx;
        let small_half_chord_left = crate::half_chord(arc_r, dx_small_left);
        let y_small_top_left = left_small_cy - small_half_chord_left;
        let y_small_bottom_left = left_small_cy + small_half_chord_left;

        let line_top_left = Path::line(
            Point::new(line_x_left, y_top_left),
            Point::new(line_x_left, y_small_top_left),
        );
        frame.stroke(&line_top_left, stroke);
        let line_bottom_left = Path::line(
            Point::new(line_x_left, y_small_bottom_left),
            Point::new(line_x_left, y_bottom_left),
        );
        frame.stroke(&line_bottom_left, stroke);

        let arc_intersect_left = (dx_small_left / arc_r).clamp(-1.0, 1.0).acos();
        let arc_path_left = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: Point::new(left_small_cx, left_small_cy),
                radius: arc_r,
                start_angle: Radians(-arc_intersect_left),
                end_angle: Radians(arc_intersect_left),
            });
        });
        frame.stroke(&arc_path_left, stroke);

        // Larger gauge arc on the left side (same center, 3x radius)
        let gauge_dx_left = dx_small_left / gauge_arc_r;
        let gauge_intersect_left = gauge_dx_left.clamp(-1.0, 1.0).acos();
        let gauge_arc_left = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: Point::new(left_small_cx, left_small_cy),
                radius: gauge_arc_r,
                start_angle: Radians(-gauge_intersect_left + gauge_margin),
                end_angle: Radians(gauge_intersect_left - gauge_margin),
            });
        });
        frame.stroke(&gauge_arc_left, stroke);

        // Ticks on the left gauge arc (0 at bottom, values increase upward)
        self.draw_arc_ticks(
            frame,
            fg,
            Point::new(left_small_cx, left_small_cy),
            gauge_arc_r,
            gauge_intersect_left - gauge_margin,
            -gauge_intersect_left + gauge_margin,
            self.left_min,
            self.left_max,
            self.left_label_every,
            full_radius,
        );

        // Arm on the left gauge
        self.draw_arm(
            frame,
            fg,
            Point::new(left_small_cx, left_small_cy),
            gauge_arc_r,
            arc_r,
            gauge_intersect_left - gauge_margin,
            -gauge_intersect_left + gauge_margin,
            self.left_min,
            self.left_max,
            self.left_value,
            full_radius,
        );

        let labels = [
            (
                &self.top_label,
                Point::new(center.x, center.y - full_radius * 0.74),
            ),
            (
                &self.right_label,
                Point::new(center.x + full_radius * 0.74, center.y),
            ),
            (
                &self.bottom_label,
                Point::new(center.x, center.y + full_radius * 0.74),
            ),
            (
                &self.left_label,
                Point::new(center.x - full_radius * 0.74, center.y),
            ),
        ];

        for (content, position) in labels {
            if !content.is_empty() {
                frame.fill_text(crate::centered_text(
                    content.clone(),
                    position,
                    font_size,
                    fg,
                    self.font,
                ));
            }
        }
    }
}

impl<Message> canvas::Program<Message> for DualGauge {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let center = frame.center();
        let full_radius = bounds.width.min(bounds.height) / 2.0;
        self.draw_at(&mut frame, theme, center, full_radius);
        vec![frame.into_geometry()]
    }
}
