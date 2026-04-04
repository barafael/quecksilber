use iced::advanced::graphics::geometry;
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::time::Instant;
use iced::widget::canvas::{Frame, Path, Stroke};
use iced::{
    Element, Event, Font, Length, Point, Rectangle, Renderer, Size, Theme, Vector, mouse, window,
};
use std::f32::consts::PI;

use geometry::Renderer as _;
use iced::advanced::Renderer as _;

/// A rotary selector widget — a small circle with labeled positions that
/// can be rotated by click-dragging up/down. An inverted triangle above
/// the circle indicates the current selection.
pub struct RotarySelector<'a, Message> {
    labels: Vec<String>,
    left_label: String,
    right_label: String,
    selected: usize,
    gap: f32,
    font: Font,
    width: Length,
    height: Length,
    on_change: Box<dyn Fn(usize) -> Message + 'a>,
}

#[derive(Debug, Clone, Copy)]
struct State {
    is_dragging: bool,
    last_y: f32,
    accumulated: f32,
    /// Current visual rotation angle (radians). Animates toward `target_rotation`.
    current_rotation: f32,
    /// Target rotation angle for the selected index.
    target_rotation: f32,
    /// Last frame timestamp for computing dt.
    last_frame: Option<Instant>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_dragging: false,
            last_y: 0.0,
            accumulated: 0.0,
            current_rotation: 0.0,
            target_rotation: 0.0,
            last_frame: None,
        }
    }
}

/// Animation speed — radians per second.
const ROTATION_SPEED: f32 = 12.0;

impl<'a, Message> RotarySelector<'a, Message> {
    /// Create a new rotary selector with the given position labels and change handler.
    pub fn new(
        labels: Vec<impl Into<String>>,
        selected: usize,
        on_change: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        let labels: Vec<String> = labels.into_iter().map(Into::into).collect();
        let selected = selected.min(labels.len().saturating_sub(1));
        Self {
            labels,
            left_label: String::new(),
            right_label: String::new(),
            selected,
            gap: 0.3,
            font: Font::default(),
            width: Length::Fill,
            height: Length::Fill,
            on_change: Box::new(on_change),
        }
    }

    pub fn left_label(mut self, label: impl Into<String>) -> Self {
        self.left_label = label.into();
        self
    }

    pub fn right_label(mut self, label: impl Into<String>) -> Self {
        self.right_label = label.into();
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index.min(self.labels.len().saturating_sub(1));
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap.clamp(0.0, 1.0);
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn set_selected(&mut self, index: usize) {
        self.selected = index.min(self.labels.len().saturating_sub(1));
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    fn angle_for_index(&self, index: usize) -> f32 {
        let n = self.labels.len();
        if n <= 1 {
            return -PI / 2.0;
        }
        let sweep = 2.0 * PI * (1.0 - self.gap);
        let start = -PI / 2.0 - sweep / 2.0;
        start + (index as f32 / (n - 1) as f32) * sweep
    }

    /// The rotation offset that places `index` at the top.
    fn rotation_for_index(&self, index: usize) -> f32 {
        -PI / 2.0 - self.angle_for_index(index)
    }

    /// Draw the rotary selector at a specific center point and radius,
    /// using the given rotation offset.
    fn draw_at_rotation(
        &self,
        frame: &mut Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
        rotation: f32,
    ) {
        let primary = theme.palette().primary;
        let fg = theme.palette().background;

        let bg = Path::circle(center, full_radius);
        frame.fill(&bg, primary);

        // -- Knurling ring: three concentric circles with 64 radial segments --
        let knurl_width = full_radius / 4.0 * 1.05;
        let knurl_outer = full_radius / 2.0 * 1.05;
        let knurl_mid = knurl_outer - knurl_width / 2.0;
        let knurl_inner = knurl_outer - knurl_width;
        let ring_stroke_w = (full_radius * 0.012).max(1.0);
        let seg_stroke_w = (full_radius * 0.008).max(0.5);

        for &r in &[knurl_inner, knurl_mid, knurl_outer] {
            frame.stroke(
                &Path::circle(center, r),
                Stroke::default().with_color(fg).with_width(ring_stroke_w),
            );
        }

        frame.with_save(|frame| {
            frame.translate(Vector::new(center.x, center.y));
            frame.rotate(rotation);
            for i in 0..64 {
                let angle = i as f32 * (2.0 * PI / 64.0);
                let cos = angle.cos();
                let sin = angle.sin();
                let p0 = Point::new(cos * knurl_mid, sin * knurl_mid);
                let p1 = Point::new(cos * knurl_outer, sin * knurl_outer);
                frame.stroke(
                    &Path::line(p0, p1),
                    Stroke::default().with_color(fg).with_width(seg_stroke_w),
                );
            }
        });

        let n = self.labels.len();
        if n == 0 {
            return;
        }

        let label_r = full_radius * 0.7;
        let label_size = full_radius * 0.18;

        for (i, label_text) in self.labels.iter().enumerate() {
            let angle = self.angle_for_index(i) + rotation;
            let pos = Point::new(
                center.x + angle.cos() * label_r,
                center.y + angle.sin() * label_r,
            );
            // Orient label along the radius: rotate so text reads outward.
            // The text rotation is angle + 90° (so baseline is tangential,
            // text points outward). Flip if the label would be upside-down.
            let text_angle = angle + PI / 2.0;
            frame.with_save(|frame| {
                frame.translate(Vector::new(pos.x, pos.y));
                frame.rotate(text_angle);
                frame.fill_text(crate::centered_text(
                    label_text.clone(),
                    Point::ORIGIN,
                    label_size,
                    fg,
                    self.font,
                ));
            });
        }

        for i in 0..n {
            let angle = self.angle_for_index(i) + rotation;
            let cos = angle.cos();
            let sin = angle.sin();
            let inner = Point::new(
                center.x + cos * full_radius * 0.88,
                center.y + sin * full_radius * 0.88,
            );
            let outer = Point::new(center.x + cos * full_radius, center.y + sin * full_radius);
            let is_selected = i == self.selected;
            let tick_w = if is_selected {
                (full_radius * 0.03).max(2.0)
            } else {
                (full_radius * 0.015).max(1.0)
            };
            frame.stroke(
                &Path::line(inner, outer),
                Stroke::default().with_color(fg).with_width(tick_w),
            );
        }

        let tri_top = full_radius * 1.15;
        let tri_bottom = full_radius * 1.02;
        let tri_half_w = full_radius * 0.12;
        let triangle = Path::new(|b| {
            b.move_to(Point::new(center.x - tri_half_w, center.y - tri_top));
            b.line_to(Point::new(center.x + tri_half_w, center.y - tri_top));
            b.line_to(Point::new(center.x, center.y - tri_bottom));
            b.close();
        });
        frame.fill(&triangle, primary);

        if !self.left_label.is_empty() {
            let pos = Point::new(
                center.x - full_radius * 0.45,
                center.y - tri_top - full_radius * 0.08,
            );
            frame.fill_text(crate::centered_text(
                self.left_label.clone(),
                pos,
                full_radius * 0.15,
                fg,
                self.font,
            ));
        }
        if !self.right_label.is_empty() {
            let pos = Point::new(
                center.x + full_radius * 0.45,
                center.y - tri_top - full_radius * 0.08,
            );
            frame.fill_text(crate::centered_text(
                self.right_label.clone(),
                pos,
                full_radius * 0.15,
                fg,
                self.font,
            ));
        }
    }

    /// Draw using the selected index rotation (no animation, for external callers).
    pub fn draw_at(&self, frame: &mut Frame, theme: &Theme, center: Point, full_radius: f32) {
        let rotation = self.rotation_for_index(self.selected);
        self.draw_at_rotation(frame, theme, center, full_radius, rotation);
    }
}

impl<Message> Widget<Message, Theme, Renderer> for RotarySelector<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let bounds = layout.bounds();

        // Sync target rotation with the current selected index.
        let target = self.rotation_for_index(self.selected);

        // On first frame, snap to the target (no animation from zero).
        if state.last_frame.is_none() {
            state.current_rotation = target;
            state.target_rotation = target;
        } else {
            state.target_rotation = target;
        }

        match event {
            // --- Animation tick ---
            Event::Window(window::Event::RedrawRequested(now)) => {
                if let Some(last) = state.last_frame {
                    let dt = now.duration_since(last).as_secs_f32().min(0.1);
                    let diff = state.target_rotation - state.current_rotation;
                    if diff.abs() > 0.001 {
                        let step = ROTATION_SPEED * dt * diff.signum();
                        if step.abs() >= diff.abs() {
                            state.current_rotation = state.target_rotation;
                        } else {
                            state.current_rotation += step;
                        }
                        shell.request_redraw();
                    }
                }
                state.last_frame = Some(*now);
            }
            // --- Drag interaction ---
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_over(bounds) {
                    state.is_dragging = true;
                    state.last_y = pos.y;
                    state.accumulated = 0.0;
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                if let Some(pos) = cursor.position() {
                    let dy = state.last_y - pos.y;
                    state.last_y = pos.y;
                    state.accumulated += dy;

                    let full_radius = bounds.width.min(bounds.height) / 2.0;
                    let n = self.labels.len();
                    if n > 1 {
                        let sweep = 2.0 * PI * (1.0 - self.gap);
                        let step_angle = sweep / (n - 1) as f32;
                        let pixels_per_step = full_radius * step_angle;

                        let steps = (state.accumulated / pixels_per_step).round() as i32;
                        if steps != 0 {
                            state.accumulated -= steps as f32 * pixels_per_step;
                            let new_index =
                                (self.selected as i32 + steps).clamp(0, n as i32 - 1) as usize;
                            if new_index != self.selected {
                                self.selected = new_index;
                                state.target_rotation = self.rotation_for_index(new_index);
                                shell.publish((self.on_change)(new_index));
                                shell.request_redraw();
                            }
                        }
                    }
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    state.accumulated = 0.0;
                }
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if bounds.width < 1.0 || bounds.height < 1.0 {
            return;
        }

        let state = tree.state.downcast_ref::<State>();
        let rotation = state.current_rotation;

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            let mut frame = Frame::new(renderer, bounds.size());
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let full_radius = bounds.width.min(bounds.height) / 2.0 * 0.7;
            self.draw_at_rotation(&mut frame, theme, center, full_radius, rotation);
            renderer.draw_geometry(frame.into_geometry());
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if state.is_dragging {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}

impl<'a, Message: 'a> From<RotarySelector<'a, Message>> for Element<'a, Message> {
    fn from(selector: RotarySelector<'a, Message>) -> Self {
        Self::new(selector)
    }
}
