use iced::Color;

use crate::indicator::IndicatorStatus;

/// Extended color constants for quecksilber widgets, beyond what iced's `Palette` provides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MercuryColors {
    /// Powered-down / inactive indicator color.
    pub off: Color,
    /// Widget panel surface background.
    pub panel_bg: Color,
    /// Gauge bezels, panel frames.
    pub bezel: Color,
    /// Gauge needles, active scan lines.
    pub needle: Color,
    /// Scale markings, tick marks.
    pub tick_mark: Color,
    /// Nominal status glow (green).
    pub nominal: Color,
    /// Caution status glow (amber).
    pub caution: Color,
    /// Alert status glow (red).
    pub alert: Color,
    /// Primary accent (cool blue).
    pub primary: Color,
    /// Text color (warm off-white).
    pub text: Color,
    /// Background color (dark blue-gray).
    pub background: Color,
}

impl MercuryColors {
    /// Returns the color associated with the given [`IndicatorStatus`].
    #[must_use]
    pub fn status_color(&self, status: IndicatorStatus) -> Color {
        match status {
            IndicatorStatus::Off => self.off,
            IndicatorStatus::Nominal => self.nominal,
            IndicatorStatus::Caution => self.caution,
            IndicatorStatus::Alert => self.alert,
        }
    }

    /// Returns the given color with modified alpha.
    #[must_use]
    pub fn with_alpha(color: Color, alpha: f32) -> Color {
        Color { a: alpha, ..color }
    }
}

impl Default for MercuryColors {
    fn default() -> Self {
        Self {
            off: Color::from_rgb8(0x3B, 0x3B, 0x3B),
            panel_bg: Color::from_rgb8(0x16, 0x1B, 0x22),
            bezel: Color::from_rgb8(0x30, 0x36, 0x3D),
            needle: Color::WHITE,
            tick_mark: Color::from_rgb8(0x6E, 0x76, 0x81),
            nominal: Color::from_rgb8(0x2E, 0xA0, 0x43),
            caution: Color::from_rgb8(0xD4, 0xA0, 0x17),
            alert: Color::from_rgb8(0xDA, 0x36, 0x33),
            primary: Color::from_rgb8(0x4A, 0x90, 0xD9),
            text: Color::from_rgb8(0xD4, 0xD4, 0xC8),
            background: Color::from_rgb8(0x0D, 0x11, 0x17),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_colors_are_opaque() {
        let colors = MercuryColors::default();
        let all = [
            colors.off,
            colors.panel_bg,
            colors.bezel,
            colors.needle,
            colors.tick_mark,
            colors.nominal,
            colors.caution,
            colors.alert,
            colors.primary,
            colors.text,
            colors.background,
        ];
        for color in all {
            assert!(
                (color.a - 1.0).abs() < f32::EPSILON,
                "Expected opaque color, got alpha={}",
                color.a
            );
        }
    }

    #[test]
    fn status_color_maps_correctly() {
        let colors = MercuryColors::default();
        assert_eq!(colors.status_color(IndicatorStatus::Off), colors.off);
        assert_eq!(
            colors.status_color(IndicatorStatus::Nominal),
            colors.nominal
        );
        assert_eq!(
            colors.status_color(IndicatorStatus::Caution),
            colors.caution
        );
        assert_eq!(colors.status_color(IndicatorStatus::Alert), colors.alert);
    }

    #[test]
    fn with_alpha_preserves_rgb() {
        let color = Color::from_rgb8(0xFF, 0x00, 0x80);
        let faded = MercuryColors::with_alpha(color, 0.5);
        assert!((faded.r - color.r).abs() < f32::EPSILON);
        assert!((faded.g - color.g).abs() < f32::EPSILON);
        assert!((faded.b - color.b).abs() < f32::EPSILON);
        assert!((faded.a - 0.5).abs() < f32::EPSILON);
    }
}
