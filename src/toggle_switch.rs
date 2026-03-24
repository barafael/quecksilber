use iced::widget::{button, column, container, row, text, toggler};
use iced::{Element, Renderer, Theme};

use crate::color::MercuryColors;

/// Creates a toggle switch with a label, styled for the Mercury theme.
///
/// Optionally includes a "guard" cover that must be opened before the
/// toggle is accessible — the safety-pin concept from the Mercury hand
/// controller.
///
/// # Arguments
///
/// * `label` - Display label for the switch.
/// * `is_on` - Current toggle state.
/// * `on_toggle` - Message produced when the toggle is flipped.
#[must_use]
pub fn toggle_switch<'a, Message: Clone + 'a>(
    label: &str,
    is_on: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> ToggleSwitch<'a, Message> {
    ToggleSwitch {
        label: label.to_string(),
        is_on,
        on_toggle: Box::new(on_toggle),
        guarded: None,
    }
}

/// A toggle switch widget builder. Created via [`toggle_switch()`].
pub struct ToggleSwitch<'a, Message> {
    label: String,
    is_on: bool,
    on_toggle: Box<dyn Fn(bool) -> Message + 'a>,
    guarded: Option<GuardConfig<'a, Message>>,
}

struct GuardConfig<'a, Message> {
    is_guarded: bool,
    on_guard_toggle: Box<dyn Fn(bool) -> Message + 'a>,
}

impl<'a, Message: Clone + 'a> ToggleSwitch<'a, Message> {
    /// Adds a guard cover to the toggle switch.
    ///
    /// When `is_guarded` is `true`, the toggle is hidden behind a red "GUARDED"
    /// overlay. Clicking the overlay fires `on_guard_toggle(false)` to reveal
    /// the actual toggle.
    #[must_use]
    pub fn guarded(
        mut self,
        is_guarded: bool,
        on_guard_toggle: impl Fn(bool) -> Message + 'a,
    ) -> Self {
        self.guarded = Some(GuardConfig {
            is_guarded,
            on_guard_toggle: Box::new(on_guard_toggle),
        });
        self
    }

    /// Converts this toggle switch into an iced `Element`.
    pub fn into_element(self) -> Element<'a, Message, Theme, Renderer> {
        let colors = MercuryColors::default();

        let label_text = text(self.label.to_uppercase())
            .size(11)
            .color(colors.text);

        // Check if guarded and guard is closed
        if let Some(guard) = &self.guarded
            && guard.is_guarded {
                let open_msg = (guard.on_guard_toggle)(false);
                let guard_cover = button(
                    text("GUARDED")
                        .size(10)
                        .color(colors.text),
                )
                .on_press(open_msg)
                .style(move |_theme: &Theme, _status| button::Style {
                    background: Some(iced::Background::Color(
                        MercuryColors::with_alpha(colors.alert, 0.6),
                    )),
                    text_color: colors.text,
                    border: iced::Border {
                        color: colors.alert,
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..button::Style::default()
                });

                return container(
                    column![label_text, guard_cover].spacing(4),
                )
                .padding(6)
                .style(move |_theme: &Theme| container::Style {
                    background: Some(iced::Background::Color(colors.panel_bg)),
                    border: iced::Border {
                        color: colors.bezel,
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    ..container::Style::default()
                })
                .into();
            }

        // Normal toggle (guard open or no guard)
        let toggle = toggler(self.is_on)
            .on_toggle(self.on_toggle)
            .size(18);

        let mut content = row![label_text, toggle].spacing(8);

        // If there's a guard, show a "LOCK" button to re-arm it
        if let Some(guard) = self.guarded {
            let lock_msg = (guard.on_guard_toggle)(true);
            let lock_btn = button(
                text("LOCK").size(9).color(colors.tick_mark),
            )
            .on_press(lock_msg)
            .padding(2)
            .style(move |_theme: &Theme, _status| button::Style {
                background: Some(iced::Background::Color(colors.bezel)),
                text_color: colors.tick_mark,
                border: iced::Border {
                    color: colors.bezel,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..button::Style::default()
            });
            content = content.push(lock_btn);
        }

        container(content)
            .padding(6)
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(colors.panel_bg)),
                border: iced::Border {
                    color: colors.bezel,
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..container::Style::default()
            })
            .into()
    }
}

impl<'a, Message: Clone + 'a> From<ToggleSwitch<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
{
    fn from(ts: ToggleSwitch<'a, Message>) -> Self {
        ts.into_element()
    }
}
