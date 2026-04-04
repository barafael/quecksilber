pub mod themes;
pub mod widgets;

/// Half-chord length of a circle: the horizontal (or vertical) distance from
/// center to the chord at perpendicular offset `offset` from center.
pub(crate) fn half_chord(radius: f32, offset: f32) -> f32 {
    (radius * radius - offset * offset).max(0.0).sqrt()
}

pub(crate) fn centered_text(
    content: String,
    position: iced::Point,
    size: f32,
    color: iced::Color,
    font: iced::Font,
) -> iced::widget::canvas::Text {
    iced::widget::canvas::Text {
        content,
        position,
        size: size.into(),
        color,
        font,
        align_x: iced::alignment::Horizontal::Center.into(),
        align_y: iced::alignment::Vertical::Center.into(),
        ..Default::default()
    }
}
