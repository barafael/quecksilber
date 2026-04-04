use iced::widget::canvas::{self, Path, Stroke};
use iced::{Element, Font, Length, Point, Rectangle, Renderer, Theme, mouse};

/// A large square attitude rate indicator with rounded corners and a dark background.
pub struct AttitudeRateIndicator {
    font: Font,
    label: String,
    yaw: f32,
    pitch: f32,
    roll: f32,
    cache: canvas::Cache,
}

impl AttitudeRateIndicator {
    pub fn new() -> Self {
        Self {
            font: Font::default(),
            label: String::new(),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            cache: canvas::Cache::new(),
        }
    }

    pub fn yaw(mut self, yaw: f32) -> Self {
        self.yaw = yaw;
        self
    }

    pub fn pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn roll(mut self, roll: f32) -> Self {
        self.roll = roll;
        self
    }

    pub fn set_yaw(&mut self, yaw: f32) {
        self.yaw = yaw;
    }

    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch;
    }

    pub fn set_roll(&mut self, roll: f32) {
        self.roll = roll;
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn view<Message: 'static>(&self) -> Element<'_, Message> {
        iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn draw_at(&self, frame: &mut canvas::Frame, theme: &Theme, center: Point, half_size: f32) {
        let primary = theme.palette().primary;
        let fg = theme.palette().background;

        let line_w = (half_size * 0.01275).max(1.0);
        let stroke = Stroke::default().with_color(fg).with_width(line_w);

        let r = half_size * 0.28;
        let x0 = center.x - half_size;
        let y0 = center.y - half_size;
        let x1 = center.x + half_size;
        let y1 = center.y + half_size;

        let rect = rounded_rect(x0, y0, x1, y1, r);

        frame.fill(&rect, primary);
        frame.stroke(&rect, stroke);

        // Circle: centered slightly right and up, diameter = 2/3 of square width
        let circle_r = half_size * 0.65;
        let circle_center = Point::new(center.x - half_size * 0.18, center.y - half_size * 0.18);
        frame.stroke(&Path::circle(circle_center, circle_r), stroke);
        let inner_r = circle_r * 0.85;
        frame.stroke(&Path::circle(circle_center, inner_r), stroke);

        // Clock-face ticks on the outer side of the inner circle (12 hour marks)
        let tick_len = (circle_r - inner_r) * 0.28;
        let tick_w = line_w * 1.4;

        for i in [9, 10, 11, 0, 1, 2, 3] {
            let angle = (i as f32) * std::f32::consts::TAU / 12.0 - std::f32::consts::FRAC_PI_2;
            let is_cardinal = i % 3 == 0;
            let tl = if is_cardinal {
                tick_len * 1.5
            } else {
                tick_len
            };
            let tw_i = if is_cardinal { tick_w * 1.3 } else { tick_w };
            let cos = angle.cos();
            let sin = angle.sin();
            let outer = Point::new(
                circle_center.x + cos * (inner_r + tl),
                circle_center.y + sin * (inner_r + tl),
            );
            let inner_pt = Point::new(
                circle_center.x + cos * inner_r,
                circle_center.y + sin * inner_r,
            );
            frame.stroke(
                &Path::line(inner_pt, outer),
                Stroke::default().with_color(fg).with_width(tw_i),
            );
        }

        // Small filled center circle
        frame.fill(&Path::circle(circle_center, circle_r * 0.12), fg);

        // Rectangle below the circle
        let left_pad = (circle_center.x - circle_r) - (center.x - half_size);
        let right_pad = half_size * 0.5;
        let rect_h = half_size * 2.0 / 6.0 * 0.7;
        let rect_top = center.y + half_size * 0.68;
        let rect_x0 = center.x - half_size + left_pad;
        let rect_x1 = center.x + half_size - right_pad;
        let bottom_rect = Path::rectangle(
            Point::new(rect_x0, rect_top),
            iced::Size::new(rect_x1 - rect_x0, rect_h),
        );
        frame.stroke(&bottom_rect, stroke);

        // Center triangles on bottom rectangle (top and bottom edges, pointing inward)
        let bot_mid_x = (rect_x0 + rect_x1) / 2.0;
        let tri_size = rect_h * 0.18;
        let tri_top = Path::new(|b| {
            b.move_to(Point::new(bot_mid_x - tri_size, rect_top));
            b.line_to(Point::new(bot_mid_x, rect_top + tri_size));
            b.line_to(Point::new(bot_mid_x + tri_size, rect_top));
            b.close();
        });
        frame.fill(&tri_top, fg);
        let tri_bot = Path::new(|b| {
            b.move_to(Point::new(bot_mid_x - tri_size, rect_top + rect_h));
            b.line_to(Point::new(bot_mid_x, rect_top + rect_h - tri_size));
            b.line_to(Point::new(bot_mid_x + tri_size, rect_top + rect_h));
            b.close();
        });
        frame.fill(&tri_bot, fg);

        // Vertical rectangle to the right of the circle
        let top_pad = (circle_center.y - circle_r) - (center.y - half_size);
        let right_rect_left = center.x + half_size * 0.60;
        let right_rect_right = right_rect_left + rect_h;
        let right_rect_top = center.y - half_size + top_pad * 0.7;
        let right_rect_bottom = rect_top - half_size * 0.08;
        let right_rect = Path::rectangle(
            Point::new(right_rect_left, right_rect_top),
            iced::Size::new(
                right_rect_right - right_rect_left,
                right_rect_bottom - right_rect_top,
            ),
        );
        frame.stroke(&right_rect, stroke);

        // Center triangles on right rectangle (left and right edges, pointing inward)
        let right_mid_y = (right_rect_top + right_rect_bottom) / 2.0;
        let tri_size_r = (right_rect_right - right_rect_left) * 0.18;
        let tri_left = Path::new(|b| {
            b.move_to(Point::new(right_rect_left, right_mid_y - tri_size_r));
            b.line_to(Point::new(right_rect_left + tri_size_r, right_mid_y));
            b.line_to(Point::new(right_rect_left, right_mid_y + tri_size_r));
            b.close();
        });
        frame.fill(&tri_left, fg);
        let tri_right = Path::new(|b| {
            b.move_to(Point::new(right_rect_right, right_mid_y - tri_size_r));
            b.line_to(Point::new(right_rect_right - tri_size_r, right_mid_y));
            b.line_to(Point::new(right_rect_right, right_mid_y + tri_size_r));
            b.close();
        });
        frame.fill(&tri_right, fg);

        let label_size = half_size * 0.08;

        // "PITCH" vertically to the left of the vertical rectangle
        let pitch_chars = ['P', 'I', 'T', 'C', 'H'];
        let char_spacing = label_size * 1.1;
        let total_height = (pitch_chars.len() - 1) as f32 * char_spacing;
        let pitch_mid_y = (right_rect_top + right_rect_bottom) / 2.0;
        let pitch_x = right_rect_left - label_size * 0.8;
        for (j, ch) in pitch_chars.iter().enumerate() {
            frame.fill_text(crate::centered_text(
                ch.to_string(),
                Point::new(
                    pitch_x,
                    pitch_mid_y - total_height / 2.0 + j as f32 * char_spacing,
                ),
                label_size,
                fg,
                self.font,
            ));
        }

        // "YAW" between circle and rectangle
        let yaw_y = (circle_center.y + circle_r + rect_top) / 2.0;
        frame.fill_text(crate::centered_text(
            "YAW".into(),
            Point::new(circle_center.x, yaw_y),
            label_size,
            fg,
            self.font,
        ));

        // "ROLL" above the circle
        frame.fill_text(crate::centered_text(
            "ROLL".into(),
            Point::new(
                circle_center.x,
                circle_center.y - circle_r - label_size * 0.8,
            ),
            label_size,
            fg,
            self.font,
        ));

        // Label below
        if !self.label.is_empty() {
            let label_pos = Point::new(center.x, center.y + half_size + half_size * 0.08);
            frame.fill_text(crate::centered_text(
                self.label.clone(),
                label_pos,
                half_size * 0.06,
                fg,
                self.font,
            ));
        }
    }
    /// Draw the roll indicator arm (dynamic, not cached).
    pub fn draw_roll_arm(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        half_size: f32,
    ) {
        let fg = theme.palette().background;

        let circle_r = half_size * 0.65;
        let circle_center = Point::new(center.x - half_size * 0.18, center.y - half_size * 0.18);
        let inner_r = circle_r * 0.85;

        // Roll input is -1.0..1.0, maps to -90°..+90° from 12 o'clock
        let roll_deg = self.roll * 90.0;
        let angle = roll_deg.to_radians() - std::f32::consts::FRAC_PI_2;
        let cos = angle.cos();
        let sin = angle.sin();
        let perp_cos = -sin;
        let perp_sin = cos;

        let half_width = circle_r * 0.025;
        let tip_length = circle_r * 0.06;
        let arm_start = circle_r * 0.15; // from center outward
        let arm_end = inner_r * 0.92; // near inner circle
        let body_end = arm_end - tip_length;

        let arm = Path::new(|b| {
            // Tail (near center)
            let tail_l = Point::new(
                circle_center.x + cos * arm_start + perp_cos * half_width,
                circle_center.y + sin * arm_start + perp_sin * half_width,
            );
            let tail_r = Point::new(
                circle_center.x + cos * arm_start - perp_cos * half_width,
                circle_center.y + sin * arm_start - perp_sin * half_width,
            );
            // Body end (before tip)
            let body_l = Point::new(
                circle_center.x + cos * body_end + perp_cos * half_width,
                circle_center.y + sin * body_end + perp_sin * half_width,
            );
            let body_r = Point::new(
                circle_center.x + cos * body_end - perp_cos * half_width,
                circle_center.y + sin * body_end - perp_sin * half_width,
            );
            // Triangular tip
            let tip = Point::new(
                circle_center.x + cos * arm_end,
                circle_center.y + sin * arm_end,
            );

            b.move_to(tail_l);
            b.line_to(body_l);
            b.line_to(tip);
            b.line_to(body_r);
            b.line_to(tail_r);
            b.close();
        });
        frame.fill(&arm, fg);

        // Small triangle at the roll arm tip, between inner and outer circles
        let tri_base = circle_r * 0.05;
        let tri_depth = (circle_r - inner_r) * 0.6;
        let tri_tip_r = inner_r; // tip touches inner circle (= arm tip radius area)
        let tri_base_r = tri_tip_r + tri_depth;
        let tri = Path::new(|b| {
            b.move_to(Point::new(
                circle_center.x + cos * tri_tip_r,
                circle_center.y + sin * tri_tip_r,
            ));
            b.line_to(Point::new(
                circle_center.x + cos * tri_base_r + perp_cos * tri_base,
                circle_center.y + sin * tri_base_r + perp_sin * tri_base,
            ));
            b.line_to(Point::new(
                circle_center.x + cos * tri_base_r - perp_cos * tri_base,
                circle_center.y + sin * tri_base_r - perp_sin * tri_base,
            ));
            b.close();
        });
        frame.fill(&tri, fg);
    }

    /// Draw the yaw tape (dynamic, not cached).
    pub fn draw_yaw_tape(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        half_size: f32,
    ) {
        let fg = theme.palette().background;

        // Recompute bottom rectangle geometry
        let circle_r = half_size * 0.65;
        let circle_center_x = center.x - half_size * 0.18;
        let left_pad = (circle_center_x - circle_r) - (center.x - half_size);
        let right_pad = half_size * 0.5;
        let rect_h = half_size * 2.0 / 6.0 * 0.7;
        let rect_top = center.y + half_size * 0.68;
        let rect_x0 = center.x - half_size + left_pad;
        let rect_x1 = center.x + half_size - right_pad;
        let rect_width = rect_x1 - rect_x0;

        let label_size = half_size * 0.09;
        let px_per_deg = rect_width / 60.0;

        let yaw = (self.yaw * 360.0).rem_euclid(360.0);

        for step in 0..36 {
            let val = step as f32 * 10.0;
            let mut diff = val - yaw;
            if diff > 180.0 {
                diff -= 360.0;
            }
            if diff < -180.0 {
                diff += 360.0;
            }

            let x = rect_x0 + rect_width / 2.0 + diff * px_per_deg;

            // Approximate half-width of a 3-digit label
            let half_w = label_size * 1.0;
            if x - half_w >= rect_x0 && x + half_w <= rect_x1 {
                let text = crate::centered_text(
                    format!("{}", val as i32),
                    Point::new(x, rect_top + rect_h / 2.0),
                    label_size,
                    fg,
                    self.font,
                );
                text.draw_with(|path, color| frame.fill(&path, color));
            }
        }
    }
    /// Draw the pitch tape (dynamic, not cached).
    pub fn draw_pitch_tape(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        half_size: f32,
    ) {
        let fg = theme.palette().background;

        // Recompute right rectangle geometry
        let circle_r = half_size * 0.65;
        let circle_center_y = center.y - half_size * 0.18;
        let rect_h = half_size * 2.0 / 6.0 * 0.7;
        let rect_top_y = center.y + half_size * 0.68;
        let top_pad = (circle_center_y - circle_r) - (center.y - half_size);
        let right_rect_left = center.x + half_size * 0.60;
        let right_rect_right = right_rect_left + rect_h;
        let right_rect_top = center.y - half_size + top_pad * 0.7;
        let right_rect_bottom = rect_top_y - half_size * 0.08;
        let right_rect_height = right_rect_bottom - right_rect_top;
        let right_rect_width = right_rect_right - right_rect_left;

        let label_size = half_size * 0.09;
        let px_per_deg = right_rect_height / 60.0;

        let pitch = (self.pitch * 90.0).clamp(-90.0, 90.0);

        for step in -9..=9 {
            let val = step as f32 * 10.0;
            let diff = val - pitch;

            let y = right_rect_top + right_rect_height / 2.0 + diff * px_per_deg;

            let half_h = label_size * 0.6;
            if y - half_h >= right_rect_top && y + half_h <= right_rect_bottom {
                let text = crate::centered_text(
                    format!("{}", (val as i32).abs()),
                    Point::new(right_rect_left + right_rect_width / 2.0, y),
                    label_size,
                    fg,
                    self.font,
                );
                text.draw_with(|path, color| frame.fill(&path, color));
            }
        }
    }
}

/// Build a rounded rectangle path using quadratic bezier approximation of arcs.
fn rounded_rect(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> Path {
    // Control point factor for approximating a quarter circle with a quadratic bezier
    // (not perfect, but close enough for small radii)
    Path::new(|b| {
        b.move_to(Point::new(x0 + r, y0));
        b.line_to(Point::new(x1 - r, y0));
        b.quadratic_curve_to(Point::new(x1, y0), Point::new(x1, y0 + r));
        b.line_to(Point::new(x1, y1 - r));
        b.quadratic_curve_to(Point::new(x1, y1), Point::new(x1 - r, y1));
        b.line_to(Point::new(x0 + r, y1));
        b.quadratic_curve_to(Point::new(x0, y1), Point::new(x0, y1 - r));
        b.line_to(Point::new(x0, y0 + r));
        b.quadratic_curve_to(Point::new(x0, y0), Point::new(x0 + r, y0));
        b.close();
    })
}

impl<Message> canvas::Program<Message> for AttitudeRateIndicator {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let center = iced::Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let half_size = bounds.width.min(bounds.height) / 2.0;

        let static_geom = self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_at(frame, theme, center, half_size);
        });
        let mut arm_frame = canvas::Frame::new(renderer, bounds.size());
        self.draw_roll_arm(&mut arm_frame, theme, center, half_size);
        self.draw_yaw_tape(&mut arm_frame, theme, center, half_size);
        self.draw_pitch_tape(&mut arm_frame, theme, center, half_size);

        vec![static_geom, arm_frame.into_geometry()]
    }
}
