use iced::widget::canvas::{self, Path};
use iced::{mouse, Element, Length, Rectangle, Renderer, Theme};

/// A filled circle in the theme's background color that fills the available space.
///
/// The circle radius is the largest that fits within the bounds.
/// The area outside the circle is transparent, showing whatever is behind the widget.
pub struct Circle;

impl Circle {
    pub fn new() -> Self {
        Self
    }

    pub fn view<Message: 'static>(&self) -> Element<'_, Message> {
        iced::widget::canvas(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl<Message> canvas::Program<Message> for Circle {
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
        let radius = bounds.width.min(bounds.height) / 2.0;
        let path = Path::circle(center, radius);
        frame.fill(&path, theme.palette().primary);
        vec![frame.into_geometry()]
    }
}
