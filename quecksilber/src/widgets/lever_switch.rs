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

/// Orientation of the lever switch track.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LeverOrientation {
    /// Lever moves left–right.
    #[default]
    Horizontal,
    /// Lever moves up–down.
    Vertical,
}

/// A lever switch widget — a hexagonal bezel with an internal crescent indicator
/// that rotates between 2 or 3 discrete positions, with text labels underneath.
pub struct LeverSwitch<'a, Message> {
    positions: usize,
    selected: usize,
    orientation: LeverOrientation,
    labels: Vec<String>,
    title: String,
    font: Font,
    width: Length,
    height: Length,
    on_change: Box<dyn Fn(usize) -> Message + 'a>,
}

#[derive(Debug, Clone, Copy)]
struct State {
    is_dragging: bool,
    last_pos: f32,
    accumulated: f32,
    /// Current animated fraction (0.0 = first position, 1.0 = last).
    current_frac: f32,
    /// Target fraction for the selected position.
    target_frac: f32,
    last_frame: Option<Instant>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            is_dragging: false,
            last_pos: 0.0,
            accumulated: 0.0,
            current_frac: 0.0,
            target_frac: 0.0,
            last_frame: None,
        }
    }
}

/// Animation speed — fraction per second (full travel in ~0.15s).
const ANIM_SPEED: f32 = 10.0;

impl<'a, Message> LeverSwitch<'a, Message> {
    /// Create a new lever switch with the given number of positions (2 or 3),
    /// the currently selected index, and a change handler.
    ///
    /// # Panics
    ///
    /// Panics if `positions` is not 2 or 3.
    pub fn new(
        positions: usize,
        selected: usize,
        on_change: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        assert!(positions == 2 || positions == 3, "positions must be 2 or 3");
        Self {
            positions,
            selected: selected.min(positions - 1),
            orientation: LeverOrientation::default(),
            labels: Vec::new(),
            title: String::new(),
            font: Font::default(),
            width: Length::Fill,
            height: Length::Fill,
            on_change: Box::new(on_change),
        }
    }

    pub fn orientation(mut self, orientation: LeverOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
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

    /// Fraction along the travel for a position index: 0.0 at first, 1.0 at last.
    fn position_fraction(&self, index: usize) -> f32 {
        if self.positions <= 1 {
            return 0.5;
        }
        index as f32 / (self.positions - 1) as f32
    }

    /// Draw the lever switch at a specific center point and radius.
    pub fn draw_at(&self, frame: &mut Frame, theme: &Theme, center: Point, full_radius: f32) {
        let frac = self.position_fraction(self.selected);
        self.draw_at_frac(frame, theme, center, full_radius, frac);
    }

    fn draw_at_frac(
        &self,
        frame: &mut Frame,
        theme: &Theme,
        center: Point,
        full_radius: f32,
        frac: f32,
    ) {
        let primary = theme.palette().primary;
        let bg = theme.palette().background;

        let is_horizontal = self.orientation == LeverOrientation::Horizontal;

        // Hexagon sizing — small relative to full_radius to leave room for labels.
        let hex_r = full_radius * 0.4;
        // Flat side on top/bottom for vertical, flat side left/right for horizontal.
        let hex_angle_offset = if is_horizontal { PI / 6.0 } else { 0.0 };
        let line_w = (hex_r * 0.03).max(1.0);

        // -- Outer hexagonal bezel (outline only) --
        let hex_outer = hexagon_path(center, hex_r, hex_angle_offset);
        frame.stroke(
            &hex_outer,
            Stroke::default().with_color(primary).with_width(line_w),
        );

        // -- Outer ring and lever base --
        let outer_ring_r = hex_r * 0.75;
        let lever_base_r = hex_r * 0.52;
        frame.stroke(
            &Path::circle(center, outer_ring_r),
            Stroke::default().with_color(primary).with_width(line_w),
        );

        // -- Lever base: white disc sitting on top, then its outline --
        frame.fill(&Path::circle(center, lever_base_r + line_w * 2.0), bg);
        frame.stroke(
            &Path::circle(center, lever_base_r),
            Stroke::default().with_color(primary).with_width(line_w),
        );

        // -- Central pivot circle (fixed at center, partially hidden by the knob) --
        let knob_r = lever_base_r * 0.6;
        let pivot_r = knob_r * 0.75;
        frame.stroke(
            &Path::circle(center, pivot_r),
            Stroke::default().with_color(primary).with_width(line_w),
        );

        // -- Knob: offset circle that slides to indicate position (1.5x stroke) --
        let knob_travel = lever_base_r * 0.52;
        let linear_offset = (frac - 0.5) * 2.0 * knob_travel;

        let knob_center = if is_horizontal {
            Point::new(center.x + linear_offset, center.y)
        } else {
            Point::new(center.x, center.y + linear_offset)
        };
        let knob_circle = Path::circle(knob_center, knob_r);
        frame.fill(&knob_circle, bg);
        frame.stroke(
            &knob_circle,
            Stroke::default()
                .with_color(primary)
                .with_width(line_w * 1.95),
        );

        // -- Title label above --
        let label_size = (full_radius * 0.2).max(8.0);
        let label_color = primary; // Labels sit outside the hex, on the panel background.
        let has_label_above =
            !self.labels.is_empty() && ((!is_horizontal && self.positions >= 2) || is_horizontal);
        if !self.title.is_empty() {
            // Push title higher when there's a label above the hex too.
            let extra = if has_label_above && !is_horizontal {
                full_radius * 0.2
            } else {
                0.0
            };
            let title_y = center.y - hex_r - full_radius * 0.15 - extra;
            frame.fill_text(crate::centered_text(
                self.title.clone(),
                Point::new(center.x, title_y),
                label_size,
                label_color,
                self.font,
            ));
        }

        // -- Position text labels --
        if !self.labels.is_empty() {
            let n = self.labels.len().min(self.positions);
            if is_horizontal && n == 2 {
                // 2-position horizontal: labels left and right of hex.
                let gap = hex_r * 0.5;
                let left = Point::new(center.x - hex_r - gap, center.y);
                let right = Point::new(center.x + hex_r + gap, center.y);
                frame.fill_text(crate::centered_text(
                    self.labels[0].clone(),
                    left,
                    label_size,
                    label_color,
                    self.font,
                ));
                frame.fill_text(crate::centered_text(
                    self.labels[1].clone(),
                    right,
                    label_size,
                    label_color,
                    self.font,
                ));
            } else if is_horizontal {
                // 3-position horizontal: labels in a row below the hex.
                let row_y = center.y + hex_r + full_radius * 0.15;
                let spread = hex_r * 2.0;
                for i in 0..n {
                    let f = self.position_fraction(i);
                    let x = center.x + (f - 0.5) * 2.0 * spread;
                    frame.fill_text(crate::centered_text(
                        self.labels[i].clone(),
                        Point::new(x, row_y),
                        label_size,
                        label_color,
                        self.font,
                    ));
                }
            } else {
                // Vertical: first label above, last label below.
                let gap = hex_r * 0.3;
                for i in 0..n {
                    let f = self.position_fraction(i);
                    let y = if f < 0.5 {
                        center.y - hex_r - gap
                    } else if f > 0.5 {
                        center.y + hex_r + gap
                    } else {
                        center.y
                    };
                    frame.fill_text(crate::centered_text(
                        self.labels[i].clone(),
                        Point::new(center.x, y),
                        label_size,
                        label_color,
                        self.font,
                    ));
                }
            }
        }
    }
}

/// Build a regular hexagon path.
fn hexagon_path(center: Point, radius: f32, angle_offset: f32) -> Path {
    Path::new(|b| {
        for i in 0..6 {
            let angle = angle_offset + i as f32 * PI / 3.0;
            let p = Point::new(
                center.x + angle.cos() * radius,
                center.y + angle.sin() * radius,
            );
            if i == 0 {
                b.move_to(p);
            } else {
                b.line_to(p);
            }
        }
        b.close();
    })
}

// ---------------------------------------------------------------------------
// Widget trait implementation (unchanged interaction logic)
// ---------------------------------------------------------------------------

impl<Message> Widget<Message, Theme, Renderer> for LeverSwitch<'_, Message> {
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
        let is_horizontal = self.orientation == LeverOrientation::Horizontal;

        // Sync target fraction with current selected index.
        let target = self.position_fraction(self.selected);
        if state.last_frame.is_none() {
            state.current_frac = target;
            state.target_frac = target;
        } else {
            state.target_frac = target;
        }

        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                if let Some(last) = state.last_frame {
                    let dt = now.duration_since(last).as_secs_f32().min(0.1);
                    let diff = state.target_frac - state.current_frac;
                    if diff.abs() > 0.001 {
                        let step = ANIM_SPEED * dt * diff.signum();
                        if step.abs() >= diff.abs() {
                            state.current_frac = state.target_frac;
                        } else {
                            state.current_frac += step;
                        }
                        shell.request_redraw();
                    }
                }
                state.last_frame = Some(*now);
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_over(bounds) {
                    state.is_dragging = true;
                    state.last_pos = if is_horizontal { pos.x } else { pos.y };
                    state.accumulated = 0.0;
                    shell.capture_event();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) if state.is_dragging => {
                if let Some(pos) = cursor.position() {
                    let cursor_axis = if is_horizontal { pos.x } else { pos.y };
                    let delta = cursor_axis - state.last_pos;
                    state.last_pos = cursor_axis;
                    state.accumulated += delta;

                    let full_radius = bounds.width.min(bounds.height) / 2.0;
                    let hex_r = full_radius * 0.7 * 0.55;
                    let slot_half = hex_r * 0.55;

                    if self.positions > 1 {
                        let step_px = (2.0 * slot_half) / (self.positions - 1) as f32;
                        let steps = (state.accumulated / step_px).round() as i32;

                        if steps != 0 {
                            state.accumulated -= steps as f32 * step_px;
                            let new_index = (self.selected as i32 + steps)
                                .clamp(0, self.positions as i32 - 1)
                                as usize;
                            if new_index != self.selected {
                                self.selected = new_index;
                                state.target_frac = self.position_fraction(new_index);
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
        let frac = state.current_frac;

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            let mut frame = Frame::new(renderer, bounds.size());
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let full_radius = bounds.width.min(bounds.height) / 2.0 * 0.7;
            self.draw_at_frac(&mut frame, theme, center, full_radius, frac);
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

impl<'a, Message: 'a> From<LeverSwitch<'a, Message>> for Element<'a, Message> {
    fn from(switch: LeverSwitch<'a, Message>) -> Self {
        Self::new(switch)
    }
}
