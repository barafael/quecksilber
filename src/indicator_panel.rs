use iced::widget::{canvas, column, container, row, text};
use iced::{Element, Renderer, Theme};

use crate::color::MercuryColors;
use crate::indicator::{Indicator, IndicatorStatus};

/// Creates a titled grid of [`Indicator`] tiles arranged in rows.
///
/// This is a view function that composes standard iced layout widgets
/// with individual `Indicator` canvas programs to form an annunciator panel,
/// inspired by the backlit indicator grids on Mercury mission control consoles.
///
/// # Arguments
///
/// * `title` - Panel title displayed at the top.
/// * `indicators` - Slice of `(label, status)` pairs.
/// * `columns` - Number of indicators per row.
#[must_use]
pub fn indicator_panel<'a, Message: 'a>(
    title: &str,
    indicators: &[(&str, IndicatorStatus)],
    columns: usize,
) -> Element<'a, Message, Theme, Renderer> {
    let colors = MercuryColors::default();
    let columns = columns.max(1);

    let title_text = text(title.to_uppercase())
        .size(12)
        .color(colors.text);

    let mut rows_vec: Vec<Element<'a, Message, Theme, Renderer>> = Vec::new();

    for chunk in indicators.chunks(columns) {
        let mut row_elements: Vec<Element<'a, Message, Theme, Renderer>> = Vec::new();

        for &(label, status) in chunk {
            let indicator = Indicator::new(label, status).colors(colors);
            let canvas_el: Element<'a, (), Theme, Renderer> =
                canvas(indicator).width(64).height(80).into();
            // Indicators produce no messages, so map () -> Message (never called).
            row_elements.push(canvas_el.map(|_: ()| unreachable!()));
        }

        rows_vec.push(
            row(row_elements)
                .spacing(2)
                .into(),
        );
    }

    let panel_content = column![title_text]
        .push(column(rows_vec).spacing(2))
        .spacing(6)
        .padding(8);

    container(panel_content)
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
