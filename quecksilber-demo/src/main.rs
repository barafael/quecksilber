mod random_walk;

use iced::widget::canvas::{self, Path};
use iced::widget::{Space, container, pick_list, stack};
use iced::{Element, Font, Length, Rectangle, Renderer, Subscription, Theme, keyboard, mouse};
use quecksilber::widgets::{
    ArmStyle, AttitudeIndicator, AttitudeRateIndicator, DualGauge, Gauge, HorizontalGauge,
    LeverOrientation, LeverSwitch, Origin, RotarySelector, Subdivision,
};
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
    split3: HorizontalGauge,
    walk_split3: RandomWalk,
    rate_indicator: AttitudeRateIndicator,
    attitude: AttitudeIndicator,
    walk_yaw: RandomWalk,
    walk_pitch: RandomWalk,
    walk_roll: RandomWalk,
    rotary_selected: usize,
    lever_h3_selected: usize,
    lever_h2_selected: usize,
    lever_v2_selected: usize,
    theme: Theme,
    themes: Vec<Theme>,
    ctrl_held: bool,
}

impl Default for State {
    fn default() -> Self {
        let cockpit = quecksilber::themes::cockpit();
        let mut themes: Vec<Theme> = Theme::ALL.to_vec();
        themes.push(cockpit.clone());

        let gauge_pressure = Gauge::new(0.0, 15.0)
            .label_every(3)
            .label("CABIN\nPRESSURE")
            .gap(0.3)
            .font(B612);
        let gauge_air = Gauge::new(0.0, 240.0)
            .label_every(20)
            .label("CABIN\nAIR")
            .gap(0.2)
            .font(B612)
            .origin(Origin::Centered)
            .subdivision(Subdivision::Every(2))
            .mid_ticks(true)
            .arm_style(ArmStyle::Needle);
        let gauge_percent = Gauge::new(0.0, 100.0)
            .label_every(20)
            .label("Percent")
            .upper_label("RELATIVE\nHUMIDITY")
            .gap(0.3)
            .font(B612)
            .subdivision(Subdivision::Every(5));
        let gauge_coolant = Gauge::new(0.0, 100.0)
            .label_every(10)
            .label("PER CENT\nREMAINING")
            .upper_label("COOLANT\nQUANTITY")
            .gap(0.3)
            .font(B612)
            .origin(Origin::Right)
            .subdivision(Subdivision::Every(5))
            .arm_style(ArmStyle::Slim);
        let gauge_co2 = Gauge::new(0.0, 4.0)
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
            .left_range(0.0, 100.0)
            .left_label_every(50)
            .right_range(0.0, 100.0)
            .right_label_every(20)
            .left_value(65.0)
            .right_value(42.0)
            .font(B612);
        Self {
            gauge_pressure,
            walk_pressure: RandomWalk::new(7.0, 0.0, 15.0, 0.03),
            gauge_air,
            walk_air: RandomWalk::new(120.0, 0.0, 240.0, 0.025),
            gauge_percent,
            walk_percent: RandomWalk::new(50.0, 0.0, 100.0, 0.03),
            gauge_coolant,
            walk_coolant: RandomWalk::new(50.0, 0.0, 100.0, 0.025),
            gauge_co2,
            walk_co2: RandomWalk::new(2.0, 0.0, 4.0, 0.03),
            dual,
            walk_dual_left: RandomWalk::new(65.0, 0.0, 100.0, 0.025),
            walk_dual_right: RandomWalk::new(42.0, 0.0, 100.0, 0.02),
            split1: HorizontalGauge::new(0.0, 30.0)
                .label_every(10)
                .tick_every(5)
                .label("DC\nVOLTS")
                .value(15.0)
                .font(B612),
            walk_split1: RandomWalk::new(15.0, 0.0, 30.0, 0.03),
            split2: HorizontalGauge::new(0.0, 50.0)
                .label_every(10)
                .label("DC\nAMPS")
                .value(25.0)
                .font(B612),
            walk_split2: RandomWalk::new(25.0, 0.0, 50.0, 0.03),
            split3: HorizontalGauge::new(0.0, 150.0)
                .label_every(50)
                .tick_every(10)
                .label("AC\nVOLTS")
                .value(75.0)
                .font(B612),
            walk_split3: RandomWalk::new(75.0, 0.0, 150.0, 0.03),
            rate_indicator: AttitudeRateIndicator::new().font(B612),
            attitude: AttitudeIndicator::new().label("ATTITUDE").font(B612),
            walk_yaw: RandomWalk::new(0.0, -60.0, 60.0, 0.02),
            walk_pitch: RandomWalk::new(0.0, -30.0, 30.0, 0.02),
            walk_roll: RandomWalk::new(0.0, -45.0, 45.0, 0.02),
            rotary_selected: 2,
            lever_h3_selected: 1,
            lever_h2_selected: 0,
            lever_v2_selected: 0,
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
    RotaryChanged(usize),
    LeverH3Changed(usize),
    LeverH2Changed(usize),
    LeverV2Changed(usize),
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
            state.split3.set_value(state.walk_split3.tick());
            let yaw = state.walk_yaw.tick();
            state.attitude.set_yaw(yaw);
            state.rate_indicator.set_yaw(yaw / 360.0);
            let pitch = state.walk_pitch.tick();
            state.attitude.set_pitch(pitch);
            state.rate_indicator.set_pitch(pitch / 90.0);
            let roll = state.walk_roll.tick();
            state.attitude.set_roll(roll);
            state.rate_indicator.set_roll(roll / 90.0);
        }
        Message::ThemeSelected(theme) => state.theme = theme,
        Message::KeyboardEvent(event) => match event {
            keyboard::Event::ModifiersChanged(modifiers) => {
                state.ctrl_held = modifiers.control();
            }
            _ => {}
        },
        Message::RotaryChanged(index) => state.rotary_selected = index,
        Message::LeverH3Changed(index) => state.lever_h3_selected = index,
        Message::LeverH2Changed(index) => state.lever_h2_selected = index,
        Message::LeverV2Changed(index) => state.lever_v2_selected = index,
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
    // Layer 1: background + gauge widgets (non-interactive, drawn on canvas)
    let main_canvas = iced::widget::canvas(GaugeLayer {
        gauge_air: &state.gauge_air,
        gauge_pressure: &state.gauge_pressure,
        gauge_percent: &state.gauge_percent,
        gauge_coolant: &state.gauge_coolant,
        gauge_co2: &state.gauge_co2,
        dual: &state.dual,
        split1: &state.split1,
        split2: &state.split2,
        split3: &state.split3,
        rate_indicator: &state.rate_indicator,
        attitude: &state.attitude,
    })
    .width(Length::Fill)
    .height(Length::Fill);

    // Layer 2: interactive widgets (real iced widgets for mouse interaction)
    let rotary: Element<'_, Message> = RotarySelector::new(
        vec!["OFF", "1", "2", "3", "BOTH"],
        state.rotary_selected,
        Message::RotaryChanged,
    )
    .left_label("L")
    .right_label("R")
    .font(B612)
    .width(Length::FillPortion(1))
    .height(Length::FillPortion(1))
    .into();

    let lever_h3: Element<'_, Message> =
        LeverSwitch::new(3, state.lever_h3_selected, Message::LeverH3Changed)
            .title("CABIN FAN")
            .labels(vec!["AUTO", "OFF", "RESET"])
            .font(B612)
            .width(Length::FillPortion(1))
            .height(Length::FillPortion(1))
            .into();

    let lever_h2: Element<'_, Message> =
        LeverSwitch::new(2, state.lever_h2_selected, Message::LeverH2Changed)
            .title("PUMP")
            .labels(vec!["ON", "OFF"])
            .font(B612)
            .width(Length::FillPortion(1))
            .height(Length::FillPortion(1))
            .into();

    let lever_v2: Element<'_, Message> =
        LeverSwitch::new(2, state.lever_v2_selected, Message::LeverV2Changed)
            .orientation(LeverOrientation::Vertical)
            .title("VALVE")
            .labels(vec!["OPEN", "SHUT"])
            .font(B612)
            .width(Length::FillPortion(1))
            .height(Length::FillPortion(1))
            .into();

    let inputs = iced::widget::column![
        iced::widget::row![rotary, Space::new()].height(Length::FillPortion(1)),
        iced::widget::row![lever_h3, lever_h2].height(Length::FillPortion(1)),
        iced::widget::row![lever_v2, Space::new()].height(Length::FillPortion(1)),
    ]
    .spacing(5)
    .padding(10)
    .width(Length::Fixed(320.0))
    .height(Length::Fill);

    let interactive_layer: Element<'_, Message> = container(inputs).into();

    // Layer 3: theme picker (only when Ctrl held)
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

    stack![main_canvas, interactive_layer, theme_picker].into()
}

struct GaugeLayer<'a> {
    gauge_air: &'a Gauge,
    gauge_pressure: &'a Gauge,
    gauge_percent: &'a Gauge,
    gauge_coolant: &'a Gauge,
    gauge_co2: &'a Gauge,
    dual: &'a DualGauge,
    split1: &'a HorizontalGauge,
    split2: &'a HorizontalGauge,
    split3: &'a HorizontalGauge,
    rate_indicator: &'a AttitudeRateIndicator,
    attitude: &'a AttitudeIndicator,
}

impl<'a> canvas::Program<Message> for GaugeLayer<'a> {
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

        // Layout: three zones side by side
        //   Left:   interactive controls (handled by iced widget layer)
        //   Center: attitude indicators
        //   Right:  gauge grid (3 cols x 4 rows)

        // Gauge sizing: fit 4 rows vertically and 3 columns in the right zone.
        // gauge_radius is derived from available height (4 rows need ~9.6 * radius)
        // and from available width (right zone ~45% of width, 3 cols need ~7.2 * radius).
        let gauge_radius = (bounds.height / 10.5).min(bounds.width * 0.45 / 7.2);
        let spacing = gauge_radius * 2.4;

        // Right zone: 3 columns of gauges
        let right_zone_width = spacing * 3.0;
        let right_zone_x = bounds.width - right_zone_width;

        // Column x-positions (within right zone)
        let col0 = right_zone_x + spacing * 0.5;
        let col1 = right_zone_x + spacing * 1.5;
        let col2 = right_zone_x + spacing * 2.5;

        // Row y-positions (vertically centered)
        let total_grid_height = spacing * 3.0;
        let grid_top = (bounds.height - total_grid_height) / 2.0;
        let row = |r: usize| grid_top + r as f32 * spacing;

        // Column 0
        self.gauge_pressure.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col0, row(0)),
            gauge_radius,
        );
        self.gauge_percent.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col0, row(1)),
            gauge_radius,
        );
        self.gauge_co2.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col0, row(2)),
            gauge_radius,
        );
        self.split1.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col0, row(3)),
            gauge_radius,
        );

        // Column 1
        self.gauge_air.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col1, row(0)),
            gauge_radius,
        );
        self.gauge_coolant.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col1, row(1)),
            gauge_radius,
        );
        self.dual.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col1, row(2)),
            gauge_radius,
        );
        self.split2.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col1, row(3)),
            gauge_radius,
        );

        // Column 2
        self.split3.draw_at(
            &mut frame,
            theme,
            iced::Point::new(col2, row(0)),
            gauge_radius,
        );

        // Attitude indicators: centered in the zone between controls and gauges.
        // The stack (attitude + rate indicator) needs ~6.8 * att_radius vertically.
        let center_zone_x = bounds.width * 0.2;
        let center_zone_width = right_zone_x - center_zone_x;
        let att_max_by_width = center_zone_width * 0.28;
        let att_max_by_height = bounds.height / 6.8;
        let att_radius = att_max_by_width.min(att_max_by_height);
        let att_center_x = center_zone_x + center_zone_width / 2.0;

        // Vertically center the entire attitude stack
        let rate_indicator_half = att_radius * 1.4;
        let gap = att_radius * 0.4;
        let total_att_height = att_radius * 2.0 + gap + rate_indicator_half * 2.0;
        let att_top = (bounds.height - total_att_height) / 2.0;
        let att_center = iced::Point::new(att_center_x, att_top + att_radius);

        // AttitudeRateIndicator: below the attitude indicator
        let rate_indicator_center_y = att_center.y + att_radius + gap + rate_indicator_half;
        let rate_indicator_center = iced::Point::new(att_center_x, rate_indicator_center_y);
        self.rate_indicator.draw_at(
            &mut frame,
            theme,
            rate_indicator_center,
            rate_indicator_half,
        );
        self.rate_indicator.draw_roll_arm(
            &mut frame,
            theme,
            rate_indicator_center,
            rate_indicator_half,
        );
        self.rate_indicator.draw_yaw_tape(
            &mut frame,
            theme,
            rate_indicator_center,
            rate_indicator_half,
        );
        self.rate_indicator.draw_pitch_tape(
            &mut frame,
            theme,
            rate_indicator_center,
            rate_indicator_half,
        );

        // Attitude indicator on top of rate_indicator
        self.attitude
            .draw_at(&mut frame, theme, att_center, att_radius);
        self.attitude
            .draw_pitch_arm(&mut frame, theme, att_center, att_radius);
        self.attitude
            .draw_roll_arm(&mut frame, theme, att_center, att_radius);
        self.attitude
            .draw_yaw_arm(&mut frame, theme, att_center, att_radius);

        vec![frame.into_geometry()]
    }
}
