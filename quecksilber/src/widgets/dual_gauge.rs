use iced::widget::canvas::{self, Path, Text};
use iced::{mouse, Element, Font, Length, Point, Radians, Rectangle, Renderer, Theme};

/// A dual-gauge widget displaying two values on a single circular face.
pub struct DualGauge {
    top_label: String,
    right_label: String,
    bottom_label: String,
    left_label: String,
    font: Font,
}

impl DualGauge {
    pub fn new() -> Self {
        Self {
            top_label: String::new(),
            right_label: String::new(),
            bottom_label: String::new(),
            left_label: String::new(),
            font: Font::default(),
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

    pub fn view<Message: 'static>(&self) -> Element<'_, Message> {
        iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
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
        let font_size = full_radius * 0.16;

        let bg_circle = Path::circle(center, full_radius);
        frame.fill(&bg_circle, primary);

        // 4 small circles on the horizontal center line
        let small_r = full_radius / 4.0;
        let positions = [
            center.x - full_radius + small_r,
            center.x - small_r,
            center.x + small_r,
        ];
        for x in positions {
            let small = Path::circle(Point::new(x, center.y), small_r);
            frame.fill(&small, fg);
        }

        // Vertical line: x at 1/5 off the rightmost point of the big circle
        let line_x = center.x + full_radius * 0.68;
        let dx_big = line_x - center.x;
        let half_chord = (full_radius * full_radius - dx_big * dx_big).max(0.0).sqrt();
        let y_top = center.y - half_chord;
        let y_bottom = center.y + half_chord;

        let warn_stroke = canvas::Stroke::default()
            .with_color(theme.palette().warning)
            .with_width((full_radius * 0.015).max(1.0));

        // Where the vertical line meets the rightmost small circle
        let right_small_cx = center.x + full_radius - small_r;
        let right_small_cy = center.y;
        let dx_small = line_x - right_small_cx;
        let small_half_chord = (small_r * small_r - dx_small * dx_small).max(0.0).sqrt();
        let y_small_top = right_small_cy - small_half_chord;
        let y_small_bottom = right_small_cy + small_half_chord;

        // Vertical line split: above and below the small circle
        let line_top = Path::line(Point::new(line_x, y_top), Point::new(line_x, y_small_top));
        frame.stroke(&line_top, warn_stroke);
        let line_bottom = Path::line(Point::new(line_x, y_small_bottom), Point::new(line_x, y_bottom));
        frame.stroke(&line_bottom, warn_stroke);

        // Arc: left half of the rightmost small circle, between vertical line intersections
        let intersect_angle = (dx_small / small_r).clamp(-1.0, 1.0).acos();
        // Top intersection is at -intersect_angle, bottom at +intersect_angle (screen coords)
        let arc_path = Path::new(|b| {
            b.arc(canvas::path::Arc {
                center: Point::new(right_small_cx, right_small_cy),
                radius: small_r,
                start_angle: Radians(intersect_angle),
                end_angle: Radians(-intersect_angle + 2.0 * std::f32::consts::PI),
            });
        });
        frame.stroke(&arc_path, warn_stroke);

        let labels = [
            (&self.top_label, Point::new(center.x, center.y - full_radius * 0.85)),
            (&self.right_label, Point::new(center.x + full_radius * 0.85, center.y)),
            (&self.bottom_label, Point::new(center.x, center.y + full_radius * 0.85)),
            (&self.left_label, Point::new(center.x - full_radius * 0.85, center.y)),
        ];

        for (content, position) in labels {
            if !content.is_empty() {
                let text = Text {
                    content: content.clone(),
                    position,
                    size: font_size.into(),
                    color: fg,
                    font: self.font,
                    align_x: iced::alignment::Horizontal::Center.into(),
                    align_y: iced::alignment::Vertical::Center.into(),
                    ..Text::default()
                };
                frame.fill_text(text);
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
