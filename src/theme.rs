use iced::theme::palette;
use iced::{Color, Theme};

use crate::color::MercuryColors;

/// The core Mercury palette mapped to iced's `Palette` fields.
pub const MERCURY_PALETTE: palette::Palette = palette::Palette {
    background: Color::from_rgb(
        0x0D as f32 / 255.0,
        0x11 as f32 / 255.0,
        0x17 as f32 / 255.0,
    ),
    text: Color::from_rgb(
        0xD4 as f32 / 255.0,
        0xD4 as f32 / 255.0,
        0xC8 as f32 / 255.0,
    ),
    primary: Color::from_rgb(
        0x4A as f32 / 255.0,
        0x90 as f32 / 255.0,
        0xD9 as f32 / 255.0,
    ),
    success: Color::from_rgb(
        0x2E as f32 / 255.0,
        0xA0 as f32 / 255.0,
        0x43 as f32 / 255.0,
    ),
    warning: Color::from_rgb(
        0xD4 as f32 / 255.0,
        0xA0 as f32 / 255.0,
        0x17 as f32 / 255.0,
    ),
    danger: Color::from_rgb(
        0xDA as f32 / 255.0,
        0x36 as f32 / 255.0,
        0x33 as f32 / 255.0,
    ),
};

/// Constructs the Mercury theme for use as an app-wide iced `Theme`.
///
/// This customizes the `Extended` palette so that standard iced widgets
/// (buttons, containers, text inputs, scrollables) look correct against
/// the dark Mercury background without per-widget style overrides.
#[must_use]
pub fn mercury_theme() -> Theme {
    Theme::custom_with_fn("Mercury", MERCURY_PALETTE, |palette| {
        palette::Extended::generate(palette)
    })
}

/// Returns the default [`MercuryColors`] for widget rendering.
#[must_use]
pub fn mercury_colors() -> MercuryColors {
    MercuryColors::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mercury_theme_uses_dark_background() {
        let theme = mercury_theme();
        let palette = theme.palette();
        assert!(
            palette.background.r < 0.1 && palette.background.g < 0.1 && palette.background.b < 0.1,
            "Expected dark background"
        );
    }

    #[test]
    fn mercury_theme_is_dark() {
        let theme = mercury_theme();
        let extended = theme.extended_palette();
        assert!(extended.is_dark, "Mercury theme should be classified as dark");
    }

    #[test]
    fn mercury_theme_has_correct_palette_values() {
        let theme = mercury_theme();
        let palette = theme.palette();
        // Primary should be blue
        assert!(palette.primary.b > palette.primary.r, "Primary should be blue-ish");
        // Danger should be red
        assert!(palette.danger.r > palette.danger.g, "Danger should be red-ish");
        // Warning should be yellow/amber
        assert!(palette.warning.r > palette.warning.b, "Warning should be amber-ish");
    }
}
