use iced::widget::canvas::{self, Path, Stroke};
use iced::{Element, Font, Length, Point, Radians, Rectangle, Renderer, Theme, mouse};

/// A circular gauge with a horizontal divider line near the lower quarter.
/// The line has a circular arc segment where it crosses an inner circle.
pub struct HorizontalGauge {
    font: Font,
    min: f32,
    max: f32,
    label_every: u32,
    tick_every: Option<u32>,
    value: f32,
    label: String,
}

impl HorizontalGauge {
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            font: Font::default(),
            min,
            max,
            label_every: 10,
            tick_every: None,
            value: 0.0,
            label: String::new(),
        }
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn label_every(mut self, n: u32) -> Self {
        self.label_every = n;
        self
    }

    pub fn tick_every(mut self, n: u32) -> Self {
        self.tick_every = Some(n);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    pub fn view<Message: 'static>(&self) -> Element<'_, Message> {
        iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn draw_at(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
    ) {
        let primary = theme.palette().primary;
        let fg = theme.palette().background;

        // Background circle
        let bg_circle = Path::circle(center, full_radius);
        frame.fill(&bg_circle, primary);

        let stroke = Stroke::default()
            .with_color(fg)
            .with_width((full_radius * 0.015).max(1.0));

        // Horizontal line at the lower quarter
        let line_y = center.y + full_radius * 0.45;

        // Where the horizontal line intersects the big circle
        let dy_big = line_y - center.y;
        let half_chord_big = crate::half_chord(full_radius, dy_big);
        let x_left = center.x - half_chord_big;
        let x_right = center.x + half_chord_big;

        // Arc center just above the bottom of the big circle, bulging upward
        let arc_center_y = center.y + full_radius * 0.72;
        let arc_r = full_radius * 0.54;

        // Where the horizontal line intersects the arc
        let dy_arc = line_y - arc_center_y;
        let inner_half_chord = crate::half_chord(arc_r, dy_arc);
        let x_inner_left = (center.x - inner_half_chord).max(x_left);
        let x_inner_right = (center.x + inner_half_chord).min(x_right);

        // Horizontal line split: left and right of the inner arc (only if there's space)
        if x_inner_left > x_left + 1.0 {
            let line_left =
                Path::line(Point::new(x_left, line_y), Point::new(x_inner_left, line_y));
            frame.stroke(&line_left, stroke);
        }

        if x_inner_right < x_right - 1.0 {
            let line_right = Path::line(
                Point::new(x_inner_right, line_y),
                Point::new(x_right, line_y),
            );
            frame.stroke(&line_right, stroke);
        }

        // Arc segment: small upper bulge between the line intersections
        let intersect_angle = ((line_y - arc_center_y) / arc_r).clamp(-1.0, 1.0).asin();
        let arc_center_pt = Point::new(center.x, arc_center_y);
        let arc_path = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: arc_center_pt,
                radius: arc_r,
                start_angle: Radians(intersect_angle),
                end_angle: Radians(-std::f32::consts::PI - intersect_angle),
            });
        });
        frame.stroke(&arc_path, stroke);

        // Larger gauge arc (center slightly above, double radius, shorter)
        let gauge_arc_r = arc_r * 2.0;
        let gauge_center_y = arc_center_y - full_radius * 0.22;
        let gauge_center_pt = Point::new(center.x, gauge_center_y);
        let gauge_margin = 0.6;
        let gauge_intersect = ((line_y - gauge_center_y) / gauge_arc_r)
            .clamp(-1.0, 1.0)
            .asin();
        let gauge_arc = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: gauge_center_pt,
                radius: gauge_arc_r,
                start_angle: Radians(gauge_intersect - gauge_margin),
                end_angle: Radians(-std::f32::consts::PI - gauge_intersect + gauge_margin),
            });
        });
        frame.stroke(&gauge_arc, stroke);

        // Ticks on the gauge arc (0 at right/bottom, max at left/bottom)
        let min = self.min;
        let max = self.max;
        let range_span = max - min;
        let label_step = self.label_every.max(1) as f32;
        let tick_step = match self.tick_every {
            Some(n) => n.max(1) as f32,
            None => label_step,
        };
        let total_ticks = (range_span / tick_step).round() as u32;

        let tick_start = -std::f32::consts::PI - gauge_intersect + gauge_margin;
        let tick_end = gauge_intersect - gauge_margin;
        let tick_sweep = tick_end - tick_start;

        let tick_len_label = gauge_arc_r * 0.10;
        let tick_len_small = gauge_arc_r * 0.05;
        let tick_w_label = (full_radius * 0.02).max(1.0);
        let tick_w_small = (full_radius * 0.01).max(0.5);
        let label_size = full_radius * 0.10;

        for i in 0..=total_ticks {
            let val = min + i as f32 * tick_step;
            if val > max + 0.01 {
                break;
            }
            let val = val.min(max);

            let t = (val - min) / range_span;
            let angle = tick_start + t * tick_sweep;
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
                gauge_center_pt.x + cos * gauge_arc_r,
                gauge_center_pt.y + sin * gauge_arc_r,
            );
            let inner = Point::new(
                gauge_center_pt.x + cos * (gauge_arc_r - tick_len),
                gauge_center_pt.y + sin * (gauge_arc_r - tick_len),
            );

            let tick = Path::line(inner, outer);
            frame.stroke(&tick, Stroke::default().with_color(fg).with_width(tick_w));

            if show_label {
                let label_r = gauge_arc_r - tick_len - gauge_arc_r * 0.12;
                let label_pos = Point::new(
                    gauge_center_pt.x + cos * label_r,
                    gauge_center_pt.y + sin * label_r,
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

        // Center label
        if !self.label.is_empty() {
            let label_pos = Point::new(center.x, center.y);
            frame.fill_text(crate::centered_text(
                self.label.clone(),
                label_pos,
                full_radius * 0.14,
                fg,
                self.font,
            ));
        }

        // Arm (blunt style, pivot at gauge arc center, starts after small arc)
        let t = ((self.value - min) / range_span).clamp(0.0, 1.0);
        let arm_angle = tick_start + t * tick_sweep;
        let cos = arm_angle.cos();
        let sin = arm_angle.sin();
        let perp_cos = -sin;
        let perp_sin = cos;

        let half_width = full_radius * 0.04;
        // Distance from gauge pivot to inner arc (approximate)
        let center_offset = (arc_center_y - gauge_center_y).abs();
        let arm_start = arc_r - center_offset;
        let arm_end = gauge_arc_r * 0.7;
        let tip_length = full_radius * 0.16;

        let arm = Path::new(|b| {
            let base_l = Point::new(
                gauge_center_pt.x + cos * arm_start + perp_cos * half_width,
                gauge_center_pt.y + sin * arm_start + perp_sin * half_width,
            );
            let base_r = Point::new(
                gauge_center_pt.x + cos * arm_start - perp_cos * half_width,
                gauge_center_pt.y + sin * arm_start - perp_sin * half_width,
            );
            let top_l = Point::new(
                gauge_center_pt.x + cos * arm_end + perp_cos * half_width,
                gauge_center_pt.y + sin * arm_end + perp_sin * half_width,
            );
            let top_r = Point::new(
                gauge_center_pt.x + cos * arm_end - perp_cos * half_width,
                gauge_center_pt.y + sin * arm_end - perp_sin * half_width,
            );
            let tip = Point::new(
                gauge_center_pt.x + cos * (arm_end + tip_length),
                gauge_center_pt.y + sin * (arm_end + tip_length),
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
}

impl<Message> canvas::Program<Message> for HorizontalGauge {
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
