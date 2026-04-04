use iced::widget::canvas::{self, Path, Stroke};
use iced::{Element, Font, Length, Point, Rectangle, Renderer, Theme, mouse};
use std::f32::consts::PI;
/// Where the scale starts on the circle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Origin {
    /// The minimum value starts at the very bottom (6 o'clock).
    #[default]
    Bottom,
    /// The gap is centered at the bottom; the scale is symmetric around the top.
    Centered,
    /// The minimum value starts at the left (9 o'clock).
    Left,
    /// The minimum value starts at the right (3 o'clock).
    Right,
}

/// How to place small subdivision ticks between labeled ticks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Subdivision {
    /// No subdivision ticks.
    None,
    /// A small tick at every integer step.
    #[default]
    Integer,
    /// A small tick every N units.
    Every(u32),
    /// Subdivide every integer step into N equal parts with small ticks.
    Fraction(u32),
}

/// The style of the gauge arm (needle).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ArmStyle {
    /// Wide rectangle with a triangular tip and short counterweight.
    #[default]
    Blunt,
    /// Thinner version of Blunt — narrower rectangle, smaller tip and counterweight.
    Slim,
    /// Thin elongated needle with a center circle and counterweight circle.
    Needle,
}

/// A circular gauge widget with tick marks, numerical labels, and an arm.
///
/// The gauge displays a value within a range on a circular dial. Zero is always
/// at the very bottom. Tick marks are drawn at each integer step, with larger
/// ticks and numerical labels at the specified label interval.
///
/// # Example
/// ```ignore
/// Gauge::new(0.0, 15.0)
///     .label_every(3)
///     .label("CABIN\nPRESSURE")
///     .gap(0.3)
///     .font(Font::with_name("B612"))
/// ```
pub struct Gauge {
    min: f32,
    max: f32,
    value: f32,
    label_every: u32,
    label: String,
    upper_label: String,
    gap: f32,
    font: Font,
    origin: Origin,
    subdivision: Subdivision,
    mid_ticks: bool,
    arm_style: ArmStyle,
    cache: canvas::Cache,
}

impl Gauge {
    /// Create a new gauge with the given min/max range.
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            value: min,
            label_every: 1,
            label: String::new(),
            upper_label: String::new(),
            gap: 0.3,
            font: Font::default(),
            origin: Origin::default(),
            subdivision: Subdivision::default(),
            mid_ticks: false,
            arm_style: ArmStyle::default(),
            cache: canvas::Cache::new(),
        }
    }

    /// Show a numerical label every N integer steps.
    pub fn label_every(mut self, n: u32) -> Self {
        self.label_every = n;
        self
    }

    /// Set the text label displayed in the lower part of the gauge.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the text label displayed in the upper part of the gauge.
    pub fn upper_label(mut self, label: impl Into<String>) -> Self {
        self.upper_label = label.into();
        self
    }

    /// Set the ratio of the full circumference left empty (0.0–1.0).
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.clamp(0.0, 1.0);
        self
    }

    /// Set how to subdivide the space between labeled ticks.
    pub fn subdivision(mut self, subdivision: Subdivision) -> Self {
        self.subdivision = subdivision;
        self
    }

    /// Show a medium tick at the midpoint between each pair of labeled ticks
    /// (only if the midpoint lands on an integer).
    pub fn mid_ticks(mut self, enabled: bool) -> Self {
        self.mid_ticks = enabled;
        self
    }

    /// Set the arm (needle) style.
    pub fn arm_style(mut self, arm_style: ArmStyle) -> Self {
        self.arm_style = arm_style;
        self
    }

    /// Set where the scale starts on the circle.
    pub fn origin(mut self, origin: Origin) -> Self {
        self.origin = origin;
        self
    }

    /// Set the font used for labels and numbers.
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Set the current value of the arm.
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    /// Draw the gauge at a specific center point and radius on the given frame.
    pub fn draw_at(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
    ) {
        let radius = full_radius * 0.85;
        let primary = theme.palette().primary;
        let fg = theme.palette().background;

        // Background circle
        let bg_circle = Path::circle(center, full_radius);
        frame.fill(&bg_circle, primary);

        let min = self.min;
        let max = self.max;
        let range_span = max - min;

        // Scale down labels and ticks when the range is large
        let label_count = (range_span / self.label_every.max(1) as f32).ceil() + 1.0;
        let density_scale = (8.0 / label_count).min(1.0).max(0.6);

        // Collect tick values: (value, TickKind)
        #[derive(PartialEq)]
        enum TickKind {
            Label,
            Mid,
            Small,
        }

        let estimated_ticks =
            (range_span / self.label_every.max(1) as f32) as usize + range_span as usize + 2;
        let mut ticks: Vec<(f32, TickKind)> = Vec::with_capacity(estimated_ticks);

        // Always add labeled ticks (at label_every intervals + endpoints)
        {
            let mut i = 0u32;
            while min + i as f32 <= max + 0.01 {
                let val = (min + i as f32).min(max);
                let is_label = (self.label_every > 0 && i % self.label_every == 0)
                    || i == 0
                    || (val - max).abs() < 0.01;
                if is_label {
                    ticks.push((val, TickKind::Label));
                }
                i += 1;
            }
        }

        // Add mid ticks (medium ticks at the midpoint between labeled ticks)
        if self.mid_ticks {
            let label_vals: Vec<f32> = ticks.iter().map(|(v, _)| *v).collect();
            for pair in label_vals.windows(2) {
                let mid = (pair[0] + pair[1]) / 2.0;
                if (mid - mid.round()).abs() < 0.01 {
                    ticks.push((mid, TickKind::Mid));
                }
            }
        }

        // Add subdivision ticks (small ticks)
        match self.subdivision {
            Subdivision::None => {}
            Subdivision::Integer => {
                let mut i = 0u32;
                while min + i as f32 <= max + 0.01 {
                    let val = (min + i as f32).min(max);
                    let already = ticks.iter().any(|(v, _)| (v - val).abs() < 0.01);
                    if !already {
                        ticks.push((val, TickKind::Small));
                    }
                    i += 1;
                }
            }
            Subdivision::Every(n) => {
                let step = n.max(1) as f32;
                let mut val = min;
                while val <= max + 0.01 {
                    let val_clamped = val.min(max);
                    let already = ticks.iter().any(|(v, _)| (v - val_clamped).abs() < 0.01);
                    if !already {
                        ticks.push((val_clamped, TickKind::Small));
                    }
                    val += step;
                }
            }
            Subdivision::Fraction(n) => {
                let n = n.max(2) as f32;
                let step = 1.0 / n;
                let total = ((max - min) * n).round() as u32;
                for i in 0..=total {
                    let val = (min + i as f32 * step).min(max);
                    let already = ticks.iter().any(|(v, _)| (v - val).abs() < step * 0.1);
                    if !already {
                        ticks.push((val, TickKind::Small));
                    }
                }
            }
        }

        // Draw all ticks
        for (val, kind) in &ticks {
            let angle = self.value_to_angle(*val);
            let show_label = *kind == TickKind::Label;

            let tick_outer = 1.0;
            let tick_inner = match kind {
                TickKind::Label => 1.0 - (1.0 - 0.83) * density_scale,
                TickKind::Mid => 1.0 - (1.0 - 0.86) * density_scale,
                TickKind::Small => 1.0 - (1.0 - 0.90) * density_scale,
            };
            let tick_width = match kind {
                TickKind::Label => (full_radius * 0.03 * density_scale).max(1.5),
                TickKind::Mid => (full_radius * 0.02 * density_scale).max(1.0),
                TickKind::Small => (full_radius * 0.012 * density_scale).max(0.5),
            };

            let cos = angle.cos();
            let sin = angle.sin();

            let inner = Point::new(
                center.x + cos * full_radius * tick_inner,
                center.y + sin * full_radius * tick_inner,
            );
            let outer = Point::new(
                center.x + cos * full_radius * tick_outer,
                center.y + sin * full_radius * tick_outer,
            );

            let tick = Path::line(inner, outer);
            frame.stroke(
                &tick,
                Stroke::default().with_color(fg).with_width(tick_width),
            );

            if show_label {
                let label_r = full_radius * (1.0 - (1.0 - 0.66) * density_scale);
                let label_pos = Point::new(center.x + cos * label_r, center.y + sin * label_r);

                frame.fill_text(crate::centered_text(
                    format!("{}", *val as i32),
                    label_pos,
                    radius * 0.2 * density_scale,
                    fg,
                    self.font,
                ));
            }
        }

        // Draw arm
        let arm_angle = self.value_to_angle(self.value);
        let cos = arm_angle.cos();
        let sin = arm_angle.sin();
        let perp_cos = -sin;
        let perp_sin = cos;

        match self.arm_style {
            ArmStyle::Blunt => {
                let half_width = full_radius * 0.05;
                let arm_length = full_radius * 0.741;
                let tip_length = full_radius * 0.095;
                let tail_length = full_radius * 0.1425;

                let arm = Path::new(|b| {
                    let base_l = Point::new(
                        center.x - cos * tail_length + perp_cos * half_width,
                        center.y - sin * tail_length + perp_sin * half_width,
                    );
                    let base_r = Point::new(
                        center.x - cos * tail_length - perp_cos * half_width,
                        center.y - sin * tail_length - perp_sin * half_width,
                    );
                    let top_l = Point::new(
                        base_l.x + cos * (arm_length + tail_length),
                        base_l.y + sin * (arm_length + tail_length),
                    );
                    let top_r = Point::new(
                        base_r.x + cos * (arm_length + tail_length),
                        base_r.y + sin * (arm_length + tail_length),
                    );
                    let tip = Point::new(
                        center.x + cos * (arm_length + tip_length),
                        center.y + sin * (arm_length + tip_length),
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
            ArmStyle::Slim => {
                let half_width = full_radius * 0.03;
                let arm_length = full_radius * 0.741;
                let tip_length = full_radius * 0.06;
                let tail_length = full_radius * 0.1;

                let arm = Path::new(|b| {
                    let base_l = Point::new(
                        center.x - cos * tail_length + perp_cos * half_width,
                        center.y - sin * tail_length + perp_sin * half_width,
                    );
                    let base_r = Point::new(
                        center.x - cos * tail_length - perp_cos * half_width,
                        center.y - sin * tail_length - perp_sin * half_width,
                    );
                    let top_l = Point::new(
                        base_l.x + cos * (arm_length + tail_length),
                        base_l.y + sin * (arm_length + tail_length),
                    );
                    let top_r = Point::new(
                        base_r.x + cos * (arm_length + tail_length),
                        base_r.y + sin * (arm_length + tail_length),
                    );
                    let tip = Point::new(
                        center.x + cos * (arm_length + tip_length),
                        center.y + sin * (arm_length + tip_length),
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
            ArmStyle::Needle => {
                let needle_width = full_radius * 0.045;
                let needle_length = full_radius * 0.72;
                let tail_length = full_radius * 0.24;
                let center_dot_r = full_radius * 0.09;
                let tail_dot_r = full_radius * 0.07;

                // Needle: rectangle body with short tapered tip
                let tip_frac = 0.35;
                let body_length = needle_length * (1.0 - tip_frac);
                let arm = Path::new(|b| {
                    let tail_end =
                        Point::new(center.x - cos * tail_length, center.y - sin * tail_length);
                    // Rectangle corners
                    let tail_l = Point::new(
                        tail_end.x + perp_cos * needle_width,
                        tail_end.y + perp_sin * needle_width,
                    );
                    let tail_r = Point::new(
                        tail_end.x - perp_cos * needle_width,
                        tail_end.y - perp_sin * needle_width,
                    );
                    let body_l = Point::new(
                        center.x + cos * body_length + perp_cos * needle_width,
                        center.y + sin * body_length + perp_sin * needle_width,
                    );
                    let body_r = Point::new(
                        center.x + cos * body_length - perp_cos * needle_width,
                        center.y + sin * body_length - perp_sin * needle_width,
                    );
                    // Pointed tip
                    let tip = Point::new(
                        center.x + cos * needle_length,
                        center.y + sin * needle_length,
                    );

                    b.move_to(tail_l);
                    b.line_to(body_l);
                    b.line_to(tip);
                    b.line_to(body_r);
                    b.line_to(tail_r);
                    b.close();
                });
                frame.fill(&arm, fg);

                // Center circle
                let center_dot = Path::circle(center, center_dot_r);
                frame.fill(&center_dot, fg);

                // Counterweight circle at the tail
                let tail_center =
                    Point::new(center.x - cos * tail_length, center.y - sin * tail_length);
                let tail_dot = Path::circle(tail_center, tail_dot_r);
                frame.fill(&tail_dot, fg);
            }
        }

        // Draw label
        let zero_label_y = center.y + full_radius * 0.78;
        let label_pos = Point::new(center.x, (center.y + zero_label_y) / 2.0);
        frame.fill_text(crate::centered_text(
            self.label.clone(),
            label_pos,
            radius * 0.18,
            fg,
            self.font,
        ));

        // Draw upper label
        if !self.upper_label.is_empty() {
            let upper_pos = Point::new(center.x, center.y - radius * 0.35);
            frame.fill_text(crate::centered_text(
                self.upper_label.clone(),
                upper_pos,
                radius * 0.18,
                fg,
                self.font,
            ));
        }
    }

    /// Draw only the static parts (background, ticks, labels) — no arm.
    fn draw_static(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
    ) {
        let radius = full_radius * 0.85;
        let primary = theme.palette().primary;
        let fg = theme.palette().background;

        let bg_circle = Path::circle(center, full_radius);
        frame.fill(&bg_circle, primary);

        let min = self.min;
        let max = self.max;
        let range_span = max - min;

        let label_count = (range_span / self.label_every.max(1) as f32).ceil() + 1.0;
        let density_scale = (8.0 / label_count).min(1.0).max(0.6);

        #[derive(PartialEq)]
        enum TickKind {
            Label,
            Mid,
            Small,
        }

        let estimated_ticks =
            (range_span / self.label_every.max(1) as f32) as usize + range_span as usize + 2;
        let mut ticks: Vec<(f32, TickKind)> = Vec::with_capacity(estimated_ticks);

        {
            let mut i = 0u32;
            while min + i as f32 <= max + 0.01 {
                let val = (min + i as f32).min(max);
                let is_label = (self.label_every > 0 && i % self.label_every == 0)
                    || i == 0
                    || (val - max).abs() < 0.01;
                if is_label {
                    ticks.push((val, TickKind::Label));
                }
                i += 1;
            }
        }

        if self.mid_ticks {
            let label_vals: Vec<f32> = ticks.iter().map(|(v, _)| *v).collect();
            for pair in label_vals.windows(2) {
                let mid = (pair[0] + pair[1]) / 2.0;
                if (mid - mid.round()).abs() < 0.01 {
                    ticks.push((mid, TickKind::Mid));
                }
            }
        }

        match self.subdivision {
            Subdivision::None => {}
            Subdivision::Integer => {
                let mut i = 0u32;
                while min + i as f32 <= max + 0.01 {
                    let val = (min + i as f32).min(max);
                    let already = ticks.iter().any(|(v, _)| (v - val).abs() < 0.01);
                    if !already {
                        ticks.push((val, TickKind::Small));
                    }
                    i += 1;
                }
            }
            Subdivision::Every(n) => {
                let step = n.max(1) as f32;
                let mut val = min;
                while val <= max + 0.01 {
                    let val_clamped = val.min(max);
                    let already = ticks.iter().any(|(v, _)| (v - val_clamped).abs() < 0.01);
                    if !already {
                        ticks.push((val_clamped, TickKind::Small));
                    }
                    val += step;
                }
            }
            Subdivision::Fraction(n) => {
                let n = n.max(2) as f32;
                let step = 1.0 / n;
                let total = ((max - min) * n).round() as u32;
                for i in 0..=total {
                    let val = (min + i as f32 * step).min(max);
                    let already = ticks.iter().any(|(v, _)| (v - val).abs() < step * 0.1);
                    if !already {
                        ticks.push((val, TickKind::Small));
                    }
                }
            }
        }

        for (val, kind) in &ticks {
            let angle = self.value_to_angle(*val);
            let show_label = *kind == TickKind::Label;

            let tick_outer = 1.0;
            let tick_inner = match kind {
                TickKind::Label => 1.0 - (1.0 - 0.83) * density_scale,
                TickKind::Mid => 1.0 - (1.0 - 0.86) * density_scale,
                TickKind::Small => 1.0 - (1.0 - 0.90) * density_scale,
            };
            let tick_width = match kind {
                TickKind::Label => (full_radius * 0.03 * density_scale).max(1.5),
                TickKind::Mid => (full_radius * 0.02 * density_scale).max(1.0),
                TickKind::Small => (full_radius * 0.012 * density_scale).max(0.5),
            };

            let cos = angle.cos();
            let sin = angle.sin();

            let inner = Point::new(
                center.x + cos * full_radius * tick_inner,
                center.y + sin * full_radius * tick_inner,
            );
            let outer = Point::new(
                center.x + cos * full_radius * tick_outer,
                center.y + sin * full_radius * tick_outer,
            );

            let tick = Path::line(inner, outer);
            frame.stroke(
                &tick,
                Stroke::default().with_color(fg).with_width(tick_width),
            );

            if show_label {
                let label_r = full_radius * (1.0 - (1.0 - 0.66) * density_scale);
                let label_pos = Point::new(center.x + cos * label_r, center.y + sin * label_r);
                frame.fill_text(crate::centered_text(
                    format!("{}", *val as i32),
                    label_pos,
                    radius * 0.2 * density_scale,
                    fg,
                    self.font,
                ));
            }
        }

        let zero_label_y = center.y + full_radius * 0.78;
        let label_pos = Point::new(center.x, (center.y + zero_label_y) / 2.0);
        frame.fill_text(crate::centered_text(
            self.label.clone(),
            label_pos,
            radius * 0.18,
            fg,
            self.font,
        ));

        if !self.upper_label.is_empty() {
            let upper_pos = Point::new(center.x, center.y - radius * 0.35);
            frame.fill_text(crate::centered_text(
                self.upper_label.clone(),
                upper_pos,
                radius * 0.18,
                fg,
                self.font,
            ));
        }
    }

    /// Draw only the arm/needle.
    fn draw_arm_only(
        &self,
        frame: &mut canvas::Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
    ) {
        let fg = theme.palette().background;
        let arm_angle = self.value_to_angle(self.value);
        let cos = arm_angle.cos();
        let sin = arm_angle.sin();
        let perp_cos = -sin;
        let perp_sin = cos;

        match self.arm_style {
            ArmStyle::Blunt => {
                let half_width = full_radius * 0.05;
                let arm_length = full_radius * 0.741;
                let tip_length = full_radius * 0.095;
                let tail_length = full_radius * 0.1425;
                let arm = Path::new(|b| {
                    let base_l = Point::new(
                        center.x - cos * tail_length + perp_cos * half_width,
                        center.y - sin * tail_length + perp_sin * half_width,
                    );
                    let base_r = Point::new(
                        center.x - cos * tail_length - perp_cos * half_width,
                        center.y - sin * tail_length - perp_sin * half_width,
                    );
                    let top_l = Point::new(
                        base_l.x + cos * (arm_length + tail_length),
                        base_l.y + sin * (arm_length + tail_length),
                    );
                    let top_r = Point::new(
                        base_r.x + cos * (arm_length + tail_length),
                        base_r.y + sin * (arm_length + tail_length),
                    );
                    let tip = Point::new(
                        center.x + cos * (arm_length + tip_length),
                        center.y + sin * (arm_length + tip_length),
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
            ArmStyle::Slim => {
                let half_width = full_radius * 0.03;
                let arm_length = full_radius * 0.741;
                let tip_length = full_radius * 0.06;
                let tail_length = full_radius * 0.1;
                let arm = Path::new(|b| {
                    let base_l = Point::new(
                        center.x - cos * tail_length + perp_cos * half_width,
                        center.y - sin * tail_length + perp_sin * half_width,
                    );
                    let base_r = Point::new(
                        center.x - cos * tail_length - perp_cos * half_width,
                        center.y - sin * tail_length - perp_sin * half_width,
                    );
                    let top_l = Point::new(
                        base_l.x + cos * (arm_length + tail_length),
                        base_l.y + sin * (arm_length + tail_length),
                    );
                    let top_r = Point::new(
                        base_r.x + cos * (arm_length + tail_length),
                        base_r.y + sin * (arm_length + tail_length),
                    );
                    let tip = Point::new(
                        center.x + cos * (arm_length + tip_length),
                        center.y + sin * (arm_length + tip_length),
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
            ArmStyle::Needle => {
                let needle_width = full_radius * 0.045;
                let needle_length = full_radius * 0.72;
                let tail_length = full_radius * 0.24;
                let center_dot_r = full_radius * 0.09;
                let tail_dot_r = full_radius * 0.07;
                let tip_frac = 0.35;
                let body_length = needle_length * (1.0 - tip_frac);
                let arm = Path::new(|b| {
                    let tail_end =
                        Point::new(center.x - cos * tail_length, center.y - sin * tail_length);
                    let tail_l = Point::new(
                        tail_end.x + perp_cos * needle_width,
                        tail_end.y + perp_sin * needle_width,
                    );
                    let tail_r = Point::new(
                        tail_end.x - perp_cos * needle_width,
                        tail_end.y - perp_sin * needle_width,
                    );
                    let body_l = Point::new(
                        center.x + cos * body_length + perp_cos * needle_width,
                        center.y + sin * body_length + perp_sin * needle_width,
                    );
                    let body_r = Point::new(
                        center.x + cos * body_length - perp_cos * needle_width,
                        center.y + sin * body_length - perp_sin * needle_width,
                    );
                    let tip = Point::new(
                        center.x + cos * needle_length,
                        center.y + sin * needle_length,
                    );
                    b.move_to(tail_l);
                    b.line_to(body_l);
                    b.line_to(tip);
                    b.line_to(body_r);
                    b.line_to(tail_r);
                    b.close();
                });
                frame.fill(&arm, fg);
                frame.fill(&Path::circle(center, center_dot_r), fg);
                let tail_center =
                    Point::new(center.x - cos * tail_length, center.y - sin * tail_length);
                frame.fill(&Path::circle(tail_center, tail_dot_r), fg);
            }
        }
    }

    pub fn view<Message: 'static>(&self) -> Element<'_, Message> {
        iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn value_to_angle(&self, val: f32) -> f32 {
        let min = self.min;
        let max = self.max;
        let t = ((val - min) / (max - min)).clamp(0.0, 1.0);

        let sweep = 2.0 * PI * (1.0 - self.gap);
        let gap_angle = 2.0 * PI * self.gap;

        // Start angle depends on origin (screen coords: Y down, angles clockwise from east)
        let start_angle = match self.origin {
            Origin::Bottom => PI / 2.0,
            Origin::Left => PI,
            Origin::Right => 0.0,
            Origin::Centered => PI / 2.0 + gap_angle / 2.0,
        };

        start_angle + t * sweep
    }
}

impl<Message> canvas::Program<Message> for Gauge {
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

        // Cached static geometry (background, ticks, labels)
        let static_geom = self.cache.draw(renderer, bounds.size(), |frame| {
            self.draw_static(frame, theme, center, full_radius);
        });

        // Dynamic geometry (arm) — redrawn every frame
        let mut arm_frame = canvas::Frame::new(renderer, bounds.size());
        self.draw_arm_only(&mut arm_frame, theme, center, full_radius);

        vec![static_geom, arm_frame.into_geometry()]
    }
}
