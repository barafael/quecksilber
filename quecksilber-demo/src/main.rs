mod random_walk;

use iced::widget::canvas::{self, Path};
use iced::widget::{container, pick_list, stack, Space};
use iced::{
    keyboard, mouse, Element, Font, Length, Rectangle, Renderer, Subscription, Theme,
};
use quecksilber::widgets::{ArmStyle, DualGauge, Gauge, Origin, HorizontalGauge, Subdivision};
use random_walk::RandomWalk;
use std::time::Duration;

const B612: Font = Font::with_name("B612");


fn main() -> iced::Result {
    iced::application(State::default, update, view)
        .title("Quecksilber Demo")
        .subscription(subscription)
        .theme(|state: &State| state.theme.clone())
        .font(include_bytes!("../../quecksilber/assets/B612-Regular.ttf").as_slice())
        .run()
}

struct State {
    gauge_pressure: Gauge,
    walk_pressure: RandomWalk,
    gauge_air: Gauge,
    walk_air: RandomWalk,
    gauge_percent: Gauge,
    walk_percent: RandomWalk,
    gauge_coolant: Gauge,
    walk_coolant: RandomWalk,
    gauge_co2: Gauge,
    walk_co2: RandomWalk,
    dual: DualGauge,
    walk_dual_left: RandomWalk,
    walk_dual_right: RandomWalk,
    split1: HorizontalGauge,
    walk_split1: RandomWalk,
    split2: HorizontalGauge,
    walk_split2: RandomWalk,
    theme: Theme,
    themes: Vec<Theme>,
    ctrl_held: bool,
}

impl Default for State {
    fn default() -> Self {
        let cockpit = quecksilber::themes::cockpit();
        let mut themes: Vec<Theme> = Theme::ALL.to_vec();
        themes.push(cockpit.clone());

        let gauge_pressure = Gauge::new(0.0..=15.0, 7.0)
            .label_every(3)
            .label("CABIN\nPRESSURE")
            .gap(0.3)
            .font(B612);
        let gauge_air = Gauge::new(0.0..=240.0, 120.0)
            .label_every(20)
            .label("CABIN\nAIR")
            .gap(0.2)
            .font(B612)
            .origin(Origin::Centered)
            .subdivision(Subdivision::Every(2))
            .mid_ticks(true)
            .arm_style(ArmStyle::Needle);
        let gauge_percent = Gauge::new(0.0..=100.0, 50.0)
            .label_every(20)
            .label("Percent")
            .upper_label("RELATIVE\nHUMIDITY")
            .gap(0.3)
            .font(B612)
            .subdivision(Subdivision::Every(5));
        let gauge_coolant = Gauge::new(0.0..=100.0, 50.0)
            .label_every(10)
            .label("PER CENT\nREMAINING")
            .upper_label("COOLANT\nQUANTITY")
            .gap(0.3)
            .font(B612)
            .origin(Origin::Right)
            .subdivision(Subdivision::Every(5))
            .arm_style(ArmStyle::Slim);
        let gauge_co2 = Gauge::new(0.0..=4.0, 2.0)
            .label_every(1)
            .label("PSI")
            .upper_label("CO2")
            .gap(0.3)
            .font(B612)
            .subdivision(Subdivision::Fraction(5));
        let dual = DualGauge::new()
            .top_label("OXYGEN")
            .right_label("RESERVE")
            .bottom_label("PERCENT\nREMAINING")
            .left_label("MAIN")
            .left_range(0.0..=100.0, 50)
            .right_range(0.0..=100.0, 20)
            .left_value(65.0)
            .right_value(42.0)
            .font(B612);
        Self {
            gauge_pressure,
            walk_pressure: RandomWalk::new(7.0, 0.0, 15.0, 0.02),
            gauge_air,
            walk_air: RandomWalk::new(120.0, 0.0, 240.0, 0.5),
            gauge_percent,
            walk_percent: RandomWalk::new(50.0, 0.0, 100.0, 0.1),
            gauge_coolant,
            walk_coolant: RandomWalk::new(50.0, 0.0, 100.0, 0.08),
            gauge_co2,
            walk_co2: RandomWalk::new(2.0, 0.0, 4.0, 0.005),
            dual,
            walk_dual_left: RandomWalk::new(65.0, 0.0, 100.0, 0.08),
            walk_dual_right: RandomWalk::new(42.0, 0.0, 100.0, 0.06),
            split1: HorizontalGauge::new(0.0..=30.0).label_every(10).tick_every(5).label("DC\nVOLTS").value(15.0).font(B612),
            walk_split1: RandomWalk::new(15.0, 0.0, 30.0, 0.05),
            split2: HorizontalGauge::new(0.0..=50.0).label_every(10).label("DC\nAMPS").value(25.0).font(B612),
            walk_split2: RandomWalk::new(25.0, 0.0, 50.0, 0.08),
            theme: cockpit,
            themes,
            ctrl_held: false,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ThemeSelected(Theme),
    KeyboardEvent(keyboard::Event),
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Tick => {
            state.gauge_pressure.set_value(state.walk_pressure.tick());
            state.gauge_air.set_value(state.walk_air.tick());
            state.gauge_percent.set_value(state.walk_percent.tick());
            state.gauge_coolant.set_value(state.walk_coolant.tick());
            state.gauge_co2.set_value(state.walk_co2.tick());
            state.dual.set_left_value(state.walk_dual_left.tick());
            state.dual.set_right_value(state.walk_dual_right.tick());
            state.split1.set_value(state.walk_split1.tick());
            state.split2.set_value(state.walk_split2.tick());
        }
        Message::ThemeSelected(theme) => state.theme = theme,
        Message::KeyboardEvent(event) => match event {
            keyboard::Event::ModifiersChanged(modifiers) => {
                state.ctrl_held = modifiers.control();
            }
            _ => {}
        },
    }
}

fn subscription(state: &State) -> Subscription<Message> {
    let tick = if !state.ctrl_held {
        iced::time::every(Duration::from_millis(16)).map(|_| Message::Tick)
    } else {
        Subscription::none()
    };
    Subscription::batch([tick, keyboard::listen().map(Message::KeyboardEvent)])
}

fn view(state: &State) -> Element<'_, Message> {
    // Layer 1: background + widgets
    let main_canvas = iced::widget::canvas(WidgetLayer {
        gauge_air: &state.gauge_air,
        gauge_pressure: &state.gauge_pressure,
        gauge_percent: &state.gauge_percent,
        gauge_coolant: &state.gauge_coolant,
        gauge_co2: &state.gauge_co2,
        dual: &state.dual,
        split1: &state.split1,
        split2: &state.split2,
    })
        .width(Length::Fill)
        .height(Length::Fill);

    // Layer 2: theme picker (only when Ctrl held)
    let theme_picker: Element<'_, Message> = if state.ctrl_held {
        container(
            pick_list(state.themes.clone(), Some(&state.theme), |theme| {
                Message::ThemeSelected(theme)
            })
            .placeholder("Choose a theme..."),
        )
        .padding(10)
        .into()
    } else {
        Space::new().into()
    };

    stack![main_canvas, theme_picker].into()
}

struct WidgetLayer<'a> {
    gauge_air: &'a Gauge,
    gauge_pressure: &'a Gauge,
    gauge_percent: &'a Gauge,
    gauge_coolant: &'a Gauge,
    gauge_co2: &'a Gauge,
    dual: &'a DualGauge,
    split1: &'a HorizontalGauge,
    split2: &'a HorizontalGauge,
}

impl<'a> canvas::Program<Message> for WidgetLayer<'a> {
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

        // Background
        let bg = Path::rectangle(iced::Point::ORIGIN, bounds.size());
        frame.fill(&bg, theme.palette().background);

        // Place gauges in the top-right, 10% of the shorter dimension
        let gauge_radius = bounds.width.min(bounds.height) * 0.075;
        let margin = gauge_radius * 1.6;
        let spacing = gauge_radius * 2.4;

        // Cabin Pressure gauge (left position)
        let center_pressure = iced::Point::new(
            bounds.width - margin - spacing,
            margin,
        );
        self.gauge_pressure.draw_at(&mut frame, theme, center_pressure, gauge_radius);

        // Cabin Air gauge (right position)
        let center_air = iced::Point::new(
            bounds.width - margin,
            margin,
        );
        self.gauge_air.draw_at(&mut frame, theme, center_air, gauge_radius);

        // Percent gauge (below Cabin Pressure)
        let center_percent = iced::Point::new(
            center_pressure.x,
            center_pressure.y + spacing,
        );
        self.gauge_percent.draw_at(&mut frame, theme, center_percent, gauge_radius);

        // Coolant gauge (below Cabin Air)
        let center_coolant = iced::Point::new(
            center_air.x,
            center_air.y + spacing,
        );
        self.gauge_coolant.draw_at(&mut frame, theme, center_coolant, gauge_radius);

        // CO2 gauge (below Percent)
        let center_co2 = iced::Point::new(
            center_percent.x,
            center_percent.y + spacing,
        );
        self.gauge_co2.draw_at(&mut frame, theme, center_co2, gauge_radius);

        // Dual gauge (right of CO2)
        let center_dual = iced::Point::new(
            center_co2.x + spacing,
            center_co2.y,
        );
        self.dual.draw_at(&mut frame, theme, center_dual, gauge_radius);

        // Split gauge 1 (below CO2/PSI)
        let center_split1 = iced::Point::new(
            center_co2.x,
            center_co2.y + spacing,
        );
        self.split1.draw_at(&mut frame, theme, center_split1, gauge_radius);

        // Split gauge 2 (below OXYGEN dual)
        let center_split2 = iced::Point::new(
            center_dual.x,
            center_dual.y + spacing,
        );
        self.split2.draw_at(&mut frame, theme, center_split2, gauge_radius);

        vec![frame.into_geometry()]
    }
}
