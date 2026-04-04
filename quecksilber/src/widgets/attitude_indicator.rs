use iced::widget::canvas::{self, Path, Stroke};
use iced::{Element, Font, Length, Point, Rectangle, Renderer, Theme, mouse};

/// An attitude indicator (artificial horizon) displaying yaw, pitch, and roll.
///
/// The instrument shows a dark circle with horizontal and vertical reference
/// lines that tilt and shift according to the aircraft's orientation.
pub struct AttitudeIndicator {
    font: Font,
    yaw: f32,
    pitch: f32,
    roll: f32,
    label: String,
    cache: canvas::Cache,
}

impl AttitudeIndicator {
    pub fn new() -> Self {
        Self {
            font: Font::default(),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            label: String::new(),
            cache: canvas::Cache::new(),
        }
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
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

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
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

        let line_w = (full_radius * 0.01275).max(1.0);
        let stroke = Stroke::default().with_color(fg).with_width(line_w);

        // Dark background circle
        frame.fill(&Path::circle(center, full_radius), primary);
        frame.stroke(&Path::circle(center, full_radius), stroke);

        // Largest inscribed square: side = radius * sqrt(2)
        let half_side = full_radius * std::f32::consts::FRAC_1_SQRT_2;

        let top_left = Point::new(center.x - half_side, center.y - half_side);
        let top_right = Point::new(center.x + half_side, center.y - half_side);
        let bot_right = Point::new(center.x + half_side, center.y + half_side);
        let bot_left = Point::new(center.x - half_side, center.y + half_side);

        // Draw only top, right, and bottom sides (no left side)
        let square_sides = Path::new(|b| {
            b.move_to(top_left);
            b.line_to(top_right); // top
            b.line_to(bot_right); // right
            b.line_to(bot_left); // bottom
        });
        frame.stroke(&square_sides, stroke);

        // Ticks on each side: 13 ticks (indices 0..=12), evenly spaced
        let tick_long = half_side * 0.10;
        let tick_short = tick_long * 0.5;
        let tick_w_major = line_w * 1.4;
        let tick_w_minor = line_w;

        // Padding: 1.3 tick-spacings inset from each end of the side
        let side_len = half_side * 2.0;
        let tick_spacing = side_len / 12.0;
        let pad = 1.3 * tick_spacing;
        let pad_frac = pad / side_len; // fraction of the side to inset

        for i in 0..=12 {
            let t = pad_frac + i as f32 / 12.0 * (1.0 - 2.0 * pad_frac);
            let is_major = i % 2 == 0;
            let tick_len = if is_major { tick_long } else { tick_short };
            let tw = if is_major { tick_w_major } else { tick_w_minor };
            let tick_stroke = Stroke::default().with_color(fg).with_width(tw);

            // Top side: ticks point inward (downward)
            let tx = top_left.x + t * (top_right.x - top_left.x);
            let ty = top_left.y;
            frame.stroke(
                &Path::line(Point::new(tx, ty), Point::new(tx, ty + tick_len)),
                tick_stroke,
            );

            // Bottom side: ticks point inward (upward)
            let bx = bot_left.x + t * (bot_right.x - bot_left.x);
            let by = bot_left.y;
            frame.stroke(
                &Path::line(Point::new(bx, by), Point::new(bx, by - tick_len)),
                tick_stroke,
            );

            // Right side: ticks point inward (leftward)
            let rx = top_right.x;
            let ry = top_right.y + t * (bot_right.y - top_right.y);
            frame.stroke(
                &Path::line(Point::new(rx, ry), Point::new(rx - tick_len, ry)),
                tick_stroke,
            );
        }

        // Side labels
        let label_size = half_side * 0.173;
        let label_gap = half_side * 0.22;

        // "ROLL" above the top side
        frame.fill_text(crate::centered_text(
            "ROLL".into(),
            Point::new(center.x, top_left.y - label_gap),
            label_size,
            fg,
            self.font,
        ));

        // "YAW" below the bottom side
        frame.fill_text(crate::centered_text(
            "YAW".into(),
            Point::new(center.x, bot_left.y + label_gap),
            label_size,
            fg,
            self.font,
        ));

        // "PITCH" vertically to the right of the right side
        let pitch_chars = ['P', 'I', 'T', 'C', 'H'];
        let char_spacing = label_size * 1.1;
        let total_height = (pitch_chars.len() - 1) as f32 * char_spacing;
        let start_y = center.y - total_height / 2.0;
        let pitch_x = top_right.x + label_gap;
        for (i, ch) in pitch_chars.iter().enumerate() {
            frame.fill_text(crate::centered_text(
                ch.to_string(),
                Point::new(pitch_x, start_y + i as f32 * char_spacing),
                label_size,
                fg,
                self.font,
            ));
        }

        // Label below the instrument
        if !self.label.is_empty() {
            let label_pos = Point::new(center.x, center.y + full_radius + full_radius * 0.14);
            frame.fill_text(crate::centered_text(
                self.label.clone(),
                label_pos,
                full_radius * 0.12,
                fg,
                self.font,
            ));
        }
    }

    /// Draw the pitch indicator arm (dynamic, not cached).
    pub fn draw_pitch_arm(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
    ) {
        let fg = theme.palette().background;
        let half_side = full_radius * std::f32::consts::FRAC_1_SQRT_2;

        // Arm is horizontal, pointing right toward the pitch side
        // Tip touches the right side of the inscribed square
        let tip_x = center.x + half_side;
        // Tail extends from center to half the radius to the left
        let tail_x = center.x - full_radius * 0.5;

        // Map pitch to vertical position along the right side's tick range
        // Pitch ticks span indices 0..=12, with padding
        let side_len = half_side * 2.0;
        let tick_spacing = side_len / 12.0;
        let pad = 1.3 * tick_spacing;
        let usable = side_len - 2.0 * pad;

        // pitch 0 = center tick (index 6), full range maps to tick 0..12
        let t = 0.5 - self.pitch / 90.0;
        let arm_y = (center.y - half_side) + pad + t * usable;

        let half_width = full_radius * 0.03;
        let tip_length = full_radius * 0.07;
        let round_r = half_width;

        // Blunt arm: rectangle body + triangular tip on right, rounded end on left
        let body_right = tip_x - tip_length;
        let arm = Path::new(|b| {
            // Start top-left, go right along top edge
            b.move_to(Point::new(tail_x, arm_y - half_width));
            b.line_to(Point::new(body_right, arm_y - half_width));
            // Triangular tip
            b.line_to(Point::new(tip_x, arm_y));
            // Back along bottom edge
            b.line_to(Point::new(body_right, arm_y + half_width));
            b.line_to(Point::new(tail_x, arm_y + half_width));
            // Rounded left cap (semicircle from bottom to top, bulging left)
            b.arc(canvas::path::Arc {
                center: Point::new(tail_x, arm_y),
                radius: round_r,
                start_angle: iced::Radians(std::f32::consts::FRAC_PI_2),
                end_angle: iced::Radians(std::f32::consts::FRAC_PI_2 + std::f32::consts::PI),
            });
            b.close();
        });
        frame.fill(&arm, fg);

        // Triangle pointer on the right side, pointing left, at the arm's y position
        let tri_base = full_radius * 0.0525;
        let tri_depth = full_radius * 0.075;
        let right_x = center.x + half_side;

        let tri = Path::new(|b| {
            b.move_to(Point::new(right_x + tri_depth, arm_y - tri_base));
            b.line_to(Point::new(right_x, arm_y));
            b.line_to(Point::new(right_x + tri_depth, arm_y + tri_base));
            b.close();
        });

        frame.fill(&tri, fg);

        // L-shaped details in all four quadrants
        let roll_t = 0.5 + self.roll / 90.0;
        let roll_arm_x = (center.x - half_side) + pad + roll_t * usable;
        let yaw_t = 0.5 + self.yaw / 90.0;
        let yaw_arm_x = (center.x - half_side) + pad + yaw_t * usable;
        let gap = full_radius * 0.03;
        let arm_gap = full_radius * 0.06;
        let line_w = (full_radius * 0.01275).max(1.0);
        let hw = line_w / 2.0;
        let l_size = full_radius * 0.25;
        let l_stroke = Stroke::default().with_color(fg).with_width(line_w);

        // Top-right: above pitch, right of roll
        let tr_x = roll_arm_x + half_width + gap + hw;
        let tr_y = arm_y - arm_gap - hw;
        let l_tr = Path::new(|b| {
            b.move_to(Point::new(tr_x, tr_y - l_size));
            b.line_to(Point::new(tr_x, tr_y));
            b.line_to(Point::new(tr_x + l_size, tr_y));
        });
        frame.stroke(&l_tr, l_stroke);

        // Top-left: above pitch, left of roll
        let tl_x = roll_arm_x - half_width - gap - hw;
        let tl_y = tr_y;
        let l_tl = Path::new(|b| {
            b.move_to(Point::new(tl_x, tl_y - l_size));
            b.line_to(Point::new(tl_x, tl_y));
            b.line_to(Point::new(tl_x - l_size, tl_y));
        });
        frame.stroke(&l_tl, l_stroke);

        // Bottom-right: below pitch, right of yaw
        let br_x = yaw_arm_x + half_width + gap + hw;
        let br_y = arm_y + arm_gap + hw;
        let l_br = Path::new(|b| {
            b.move_to(Point::new(br_x, br_y + l_size));
            b.line_to(Point::new(br_x, br_y));
            b.line_to(Point::new(br_x + l_size, br_y));
        });
        frame.stroke(&l_br, l_stroke);

        // Bottom-left: below pitch, left of yaw
        let bl_x = yaw_arm_x - half_width - gap - hw;
        let bl_y = br_y;
        let l_bl = Path::new(|b| {
            b.move_to(Point::new(bl_x, bl_y + l_size));
            b.line_to(Point::new(bl_x, bl_y));
            b.line_to(Point::new(bl_x - l_size, bl_y));
        });
        frame.stroke(&l_bl, l_stroke);
    }

    /// Draw the roll indicator arm (dynamic, not cached).
    pub fn draw_roll_arm(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
    ) {
        let fg = theme.palette().background;
        let half_side = full_radius * std::f32::consts::FRAC_1_SQRT_2;

        // Map roll to horizontal position along the top side's tick range
        let side_len = half_side * 2.0;
        let tick_spacing = side_len / 12.0;
        let pad = 1.3 * tick_spacing;
        let usable = side_len - 2.0 * pad;

        // roll 0 = center tick (index 6), full range maps to tick 0..12
        let t = 0.5 + self.roll / 90.0;
        let arm_x = (center.x - half_side) + pad + t * usable;

        // Compute pitch arm y to start 4px above it
        let pitch_t = 0.5 - self.pitch / 90.0;
        let pitch_arm_y = (center.y - half_side) + pad + pitch_t * usable;
        let arm_bottom = pitch_arm_y - full_radius * 0.06;

        // Tip touches the top side of the inscribed square
        let tip_y = center.y - half_side;
        // Tail extends downward from pitch arm position
        let tail_y = arm_bottom;

        let half_width = full_radius * 0.03;
        let tip_length = full_radius * 0.07;

        // Blunt arm: rectangle body + triangular tip on top, flat end on bottom
        let body_top = tip_y + tip_length;
        let arm = Path::new(|b| {
            b.move_to(Point::new(arm_x - half_width, tail_y));
            b.line_to(Point::new(arm_x - half_width, body_top));
            b.line_to(Point::new(arm_x, tip_y));
            b.line_to(Point::new(arm_x + half_width, body_top));
            b.line_to(Point::new(arm_x + half_width, tail_y));
            b.close();
        });
        frame.fill(&arm, fg);

        // Triangle pointer on the top side, pointing down, at the arm's x position
        let tri_base = full_radius * 0.0525;
        let tri_depth = full_radius * 0.075;
        let top_y = center.y - half_side;

        let tri = Path::new(|b| {
            b.move_to(Point::new(arm_x - tri_base, top_y - tri_depth));
            b.line_to(Point::new(arm_x, top_y));
            b.line_to(Point::new(arm_x + tri_base, top_y - tri_depth));
            b.close();
        });
        frame.fill(&tri, fg);
    }

    /// Draw the yaw indicator arm (dynamic, not cached).
    pub fn draw_yaw_arm(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
    ) {
        let fg = theme.palette().background;
        let half_side = full_radius * std::f32::consts::FRAC_1_SQRT_2;

        // Map yaw to horizontal position along the bottom side's tick range
        let side_len = half_side * 2.0;
        let tick_spacing = side_len / 12.0;
        let pad = 1.3 * tick_spacing;
        let usable = side_len - 2.0 * pad;

        // yaw 0 = center tick (index 6), full range maps to tick 0..12
        let t = 0.5 + self.yaw / 90.0;
        let arm_x = (center.x - half_side) + pad + t * usable;

        // Compute pitch arm y to start below it with a gap
        let pitch_t = 0.5 - self.pitch / 90.0;
        let pitch_arm_y = (center.y - half_side) + pad + pitch_t * usable;
        let arm_top = pitch_arm_y + full_radius * 0.06;

        // Tip touches the bottom side of the inscribed square
        let tip_y = center.y + half_side;
        let tail_y = arm_top;

        let half_width = full_radius * 0.03;
        let tip_length = full_radius * 0.07;

        // Blunt arm: rectangle body + triangular tip on bottom, flat end on top
        let body_bottom = tip_y - tip_length;
        let arm = Path::new(|b| {
            b.move_to(Point::new(arm_x - half_width, tail_y));
            b.line_to(Point::new(arm_x - half_width, body_bottom));
            b.line_to(Point::new(arm_x, tip_y));
            b.line_to(Point::new(arm_x + half_width, body_bottom));
            b.line_to(Point::new(arm_x + half_width, tail_y));
            b.close();
        });
        frame.fill(&arm, fg);

        // Triangle pointer on the bottom side, pointing up, at the arm's x position
        let tri_base = full_radius * 0.0525;
        let tri_depth = full_radius * 0.075;
        let bot_y = center.y + half_side;

        let tri = Path::new(|b| {
            b.move_to(Point::new(arm_x - tri_base, bot_y + tri_depth));
            b.line_to(Point::new(arm_x, bot_y));
            b.line_to(Point::new(arm_x + tri_base, bot_y + tri_depth));
            b.close();
        });
        frame.fill(&tri, fg);
    }
}

impl<Message> canvas::Program<Message> for AttitudeIndicator {
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
        let full_radius = bounds.width.min(bounds.height) / 2.0;

        let static_geom = self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_at(frame, theme, center, full_radius);
        });

        let mut arm_frame = canvas::Frame::new(renderer, bounds.size());
        self.draw_pitch_arm(&mut arm_frame, theme, center, full_radius);
        self.draw_roll_arm(&mut arm_frame, theme, center, full_radius);
        self.draw_yaw_arm(&mut arm_frame, theme, center, full_radius);

        vec![static_geom, arm_frame.into_geometry()]
    }
}
