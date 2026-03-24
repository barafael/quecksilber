use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Event, Frame, Geometry, Path, Stroke, Text};
use iced::{Point, Rectangle, Renderer, Theme};

use std::f32::consts::{FRAC_PI_2, TAU};

use crate::anim::Spring;
use crate::color::MercuryColors;
use crate::draw;

/// A rotary selector dial for choosing between discrete labeled options.
///
/// Inspired by the rotary knobs on Mercury console side panels used for
/// mode selection. Options are arranged around the circumference; a pointer
/// indicates the current selection. Mouse drag rotates to snap to the nearest
/// option.
pub struct SelectorDial<Message: Clone> {
    selected: usize,
    options: Vec<String>,
    on_select: Box<dyn Fn(usize) -> Message>,
    colors: MercuryColors,
}

impl<Message: Clone> std::fmt::Debug for SelectorDial<Message> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectorDial")
            .field("selected", &self.selected)
            .field("options", &self.options)
            .finish_non_exhaustive()
    }
}

impl<Message: Clone> SelectorDial<Message> {
    /// Creates a new selector dial.
    #[must_use]
    pub fn new(
        options: &[&str],
        selected: usize,
        on_select: impl Fn(usize) -> Message + 'static,
    ) -> Self {
        Self {
            selected: selected.min(options.len().saturating_sub(1)),
            options: options.iter().map(|s| s.to_string()).collect(),
            on_select: Box::new(on_select),
            colors: MercuryColors::default(),
        }
    }

    /// Sets a custom color palette.
    #[must_use]
    pub fn colors(mut self, colors: MercuryColors) -> Self {
        self.colors = colors;
        self
    }

    fn option_angle(&self, index: usize) -> f32 {
        if self.options.is_empty() {
            return -FRAC_PI_2;
        }
        // Distribute options evenly starting from 12 o'clock (-π/2)
        -FRAC_PI_2 + (index as f32 / self.options.len() as f32) * TAU
    }

    fn angle_to_index(&self, angle: f32) -> usize {
        if self.options.is_empty() {
            return 0;
        }
        let n = self.options.len();
        // Normalize angle to [0, TAU)
        let normalized = (angle + FRAC_PI_2).rem_euclid(TAU);
        let sector_size = TAU / n as f32;
        
        ((normalized + sector_size / 2.0) / sector_size) as usize % n
    }
}

/// Internal state for the selector dial's animation and interaction.
#[derive(Debug)]
pub struct SelectorDialState {
    pointer_spring: Spring,
    dragging: bool,
}

impl Default for SelectorDialState {
    fn default() -> Self {
        Self {
            // Underdamped spring — rotational snap with slight oscillation
            pointer_spring: Spring::with_params(-FRAC_PI_2, 250.0, 0.7),
            dragging: false,
        }
    }
}

impl<Message: Clone + 'static> canvas::Program<Message, Theme, Renderer> for SelectorDial<Message> {
    type State = SelectorDialState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);

        // Update spring target to selected option angle
        let target = self.option_angle(self.selected);
        state.pointer_spring.set_target(target);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(pos) = cursor.position_in(bounds) {
                    let dx = pos.x - center.x;
                    let dy = pos.y - center.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let outer_radius = (bounds.width.min(bounds.height) / 2.0) - 4.0;
                    if dist < outer_radius {
                        state.dragging = true;
                        let angle = dy.atan2(dx);
                        let new_index = self.angle_to_index(angle);
                        if new_index != self.selected {
                            return Some(canvas::Action::publish((self.on_select)(new_index)));
                        }
                        return Some(canvas::Action::capture());
                    }
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.dragging => {
                    state.dragging = false;
                    return Some(canvas::Action::capture());
                }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.dragging
                    && let Some(pos) = cursor.position_in(bounds) {
                        let dx = pos.x - center.x;
                        let dy = pos.y - center.y;
                        let angle = dy.atan2(dx);
                        let new_index = self.angle_to_index(angle);
                        if new_index != self.selected {
                            return Some(canvas::Action::publish((self.on_select)(new_index)));
                        }
                    }
            }
            _ => {}
        }

        if !state.pointer_spring.is_settled() {
            state.pointer_spring.tick(1.0 / 60.0);
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
        let mut frame = Frame::new(renderer, bounds.size());
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let outer_radius = (bounds.width.min(bounds.height) / 2.0) - 4.0;
        let bezel_radius = outer_radius * 0.65; // smaller bezel leaves room for labels outside
        let knob_radius = outer_radius * 0.30;
        let label_radius = outer_radius * 0.86; // labels outside the bezel
        let dot_radius_pos = outer_radius * 0.72; // dots between bezel and labels

        // Background fill (full area)
        let bg = Path::circle(center, outer_radius);
        frame.fill(&bg, self.colors.panel_bg);

        // Outer bezel ring
        draw::draw_bezel(&mut frame, center, bezel_radius, self.colors.bezel, 1.5);

        // Option dots and labels (outside the bezel)
        for (i, option) in self.options.iter().enumerate() {
            let angle = self.option_angle(i);
            let (sin, cos) = angle.sin_cos();

            let color = if i == self.selected {
                self.colors.primary
            } else {
                self.colors.tick_mark
            };

            // Position dot
            let dot_pos = Point::new(
                center.x + cos * dot_radius_pos,
                center.y + sin * dot_radius_pos,
            );
            let dot = Path::circle(dot_pos, 3.0);
            frame.fill(&dot, color);

            // Label — anchored so text grows away from center to avoid clipping.
            let label_pos = Point::new(
                center.x + cos * label_radius,
                center.y + sin * label_radius,
            );
            let align_x = if cos > 0.3 {
                iced::alignment::Horizontal::Left
            } else if cos < -0.3 {
                iced::alignment::Horizontal::Right
            } else {
                iced::alignment::Horizontal::Center
            };
            let align_y = if sin > 0.3 {
                iced::alignment::Vertical::Top
            } else if sin < -0.3 {
                iced::alignment::Vertical::Bottom
            } else {
                iced::alignment::Vertical::Center
            };

            frame.fill_text(Text {
                content: option.clone(),
                position: label_pos,
                color,
                size: iced::Pixels(10.0),
                align_x: align_x.into(),
                align_y,
                ..Text::default()
            });
        }

        // Knob circle (center)
        let knob = Path::circle(center, knob_radius);
        frame.fill(&knob, self.colors.bezel);
        frame.stroke(
            &knob,
            Stroke::default()
                .with_color(self.colors.tick_mark)
                .with_width(1.0),
        );

        // Pointer line (animated) — extends from knob center to the dot ring
        let pointer_angle = state.pointer_spring.value();
        let (sin, cos) = pointer_angle.sin_cos();
        let pointer_start = Point::new(
            center.x + cos * (knob_radius * 0.3),
            center.y + sin * (knob_radius * 0.3),
        );
        let pointer_end = Point::new(
            center.x + cos * dot_radius_pos,
            center.y + sin * dot_radius_pos,
        );
        let pointer = Path::line(pointer_start, pointer_end);
        frame.stroke(
            &pointer,
            Stroke::default()
                .with_color(self.colors.primary)
                .with_width(2.5),
        );

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.dragging {
            return mouse::Interaction::Grabbing;
        }
        if let Some(pos) = cursor.position_in(bounds) {
            let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
            let dx = pos.x - center.x;
            let dy = pos.y - center.y;
            let dist = (dx * dx + dy * dy).sqrt();
            let outer_radius = (bounds.width.min(bounds.height) / 2.0) - 4.0;
            if dist < outer_radius {
                return mouse::Interaction::Pointer;
            }
        }
        mouse::Interaction::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_to_index_basic() {
        let dial: SelectorDial<()> = SelectorDial {
            selected: 0,
            options: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            on_select: Box::new(|_| ()),
            colors: MercuryColors::default(),
        };

        // 12 o'clock should be index 0
        let idx = dial.angle_to_index(-FRAC_PI_2);
        assert_eq!(idx, 0);
    }

    #[test]
    fn angle_to_index_wraps() {
        let dial: SelectorDial<()> = SelectorDial {
            selected: 0,
            options: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            on_select: Box::new(|_| ()),
            colors: MercuryColors::default(),
        };

        // 3 o'clock = 0 radians should be index 1 (quarter turn from 12)
        let idx = dial.angle_to_index(0.0);
        assert_eq!(idx, 1);
    }

    #[test]
    fn option_angle_first_is_twelve_oclock() {
        let dial: SelectorDial<()> = SelectorDial {
            selected: 0,
            options: vec!["A".into(), "B".into()],
            on_select: Box::new(|_| ()),
            colors: MercuryColors::default(),
        };

        let angle = dial.option_angle(0);
        assert!((angle - (-FRAC_PI_2)).abs() < f32::EPSILON);
    }

    #[test]
    fn empty_options_does_not_panic() {
        let dial: SelectorDial<()> = SelectorDial {
            selected: 0,
            options: vec![],
            on_select: Box::new(|_| ()),
            colors: MercuryColors::default(),
        };

        assert_eq!(dial.angle_to_index(0.0), 0);
        let _ = dial.option_angle(0);
    }
}
