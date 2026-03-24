use std::time::Duration;

use iced::widget::{canvas, column, container, row, text, Space};
use iced::{Element, Length, Subscription, Task};
use rand::Rng;

use quecksilber::{
    async_action_button, guarded_button, indicator_panel, mercury_theme, toggle_switch,
    ActionState, AttitudeIndicator, EventTimer, Gauge, GuardState, IndicatorStatus, MercuryColors,
    SelectorDial, StatusBar,
};

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title("Quecksilber Demo")
        .theme(mercury_theme())
        .subscription(App::subscription)
        .window_size((1200.0, 800.0))
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ToggleGyro(bool),
    ToggleComms(bool),
    ToggleLights(bool),
    ToggleRetro(bool),
    RetroGuard(bool),
    AbortGuard(GuardState),
    AbortExecute,
    FlightMode(usize),
    AsyncAction1,
    AsyncAction1Complete,
}

struct App {
    elapsed: Duration,
    systems: Vec<(&'static str, IndicatorStatus)>,
    abort_sequence: Vec<(&'static str, IndicatorStatus)>,
    gyro_on: bool,
    comms_on: bool,
    lights_on: bool,
    retro_on: bool,
    retro_guarded: bool,
    abort_state: GuardState,
    flight_mode: usize,
    cabin_pressure: f32,
    o2_supply: f32,
    dc_volts: f32,
    attitude_rate: f32,
    roll: f32,
    pitch: f32,
    yaw: f32,
    fuel_remaining: f32,
    battery: f32,
    signal: f32,
    action1_state: ActionState,
    action1_started: Option<Duration>,
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        (
            Self {
                elapsed: Duration::ZERO,
                systems: vec![
                    ("ELEC", IndicatorStatus::Nominal),
                    ("COMMS", IndicatorStatus::Nominal),
                    ("O2", IndicatorStatus::Nominal),
                    ("CABIN", IndicatorStatus::Nominal),
                    ("RETRO", IndicatorStatus::Off),
                    ("GYRO", IndicatorStatus::Nominal),
                    ("ENVIRO", IndicatorStatus::Nominal),
                    ("STAB", IndicatorStatus::Caution),
                    ("FUEL", IndicatorStatus::Nominal),
                    ("HEAT", IndicatorStatus::Alert),
                    ("RCS", IndicatorStatus::Nominal),
                    ("TELM", IndicatorStatus::Nominal),
                ],
                abort_sequence: vec![
                    ("TOWER", IndicatorStatus::Off),
                    ("SEP", IndicatorStatus::Off),
                    ("RETRO", IndicatorStatus::Off),
                    ("CHUTE", IndicatorStatus::Off),
                ],
                gyro_on: true,
                comms_on: true,
                lights_on: false,
                retro_on: false,
                retro_guarded: true,
                abort_state: GuardState::Armed,
                flight_mode: 0,
                cabin_pressure: 14.2,
                o2_supply: 87.0,
                dc_volts: 27.5,
                attitude_rate: 1.5,
                roll: 2.3,
                pitch: -1.5,
                yaw: 0.8,
                fuel_remaining: 72.0,
                battery: 91.0,
                signal: 85.0,
                action1_state: ActionState::Idle,
                action1_started: None,
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                self.elapsed += Duration::from_millis(100);

                // Slow random walk for gauge values
                let mut rng = rand::thread_rng();
                let walk =
                    |rng: &mut rand::rngs::ThreadRng, v: f32, step: f32, lo: f32, hi: f32| {
                        (v + rng.gen_range(-step..=step)).clamp(lo, hi)
                    };
                self.cabin_pressure = walk(&mut rng, self.cabin_pressure, 0.2, 10.0, 19.0);
                self.o2_supply = walk(&mut rng, self.o2_supply, 0.4, 5.0, 99.0);
                self.dc_volts = walk(&mut rng, self.dc_volts, 0.12, 22.0, 31.0);
                self.attitude_rate = walk(&mut rng, self.attitude_rate, 0.15, -5.0, 5.0);
                self.roll = walk(&mut rng, self.roll, 0.5, -30.0, 30.0);
                self.pitch = walk(&mut rng, self.pitch, 0.3, -20.0, 20.0);
                self.yaw = walk(&mut rng, self.yaw, 0.5, -30.0, 30.0);
                self.fuel_remaining = walk(&mut rng, self.fuel_remaining, 0.3, 5.0, 95.0);
                self.battery = walk(&mut rng, self.battery, 0.25, 10.0, 98.0);
                self.signal = walk(&mut rng, self.signal, 1.2, 10.0, 98.0);

                // Occasionally flip a random system indicator (~5% per tick)
                if rng.gen_ratio(1, 20) {
                    let idx = rng.gen_range(0..self.systems.len());
                    self.systems[idx].1 = match rng.gen_range(0u8..4) {
                        0 => IndicatorStatus::Nominal,
                        1 => IndicatorStatus::Caution,
                        2 => IndicatorStatus::Alert,
                        _ => IndicatorStatus::Off,
                    };
                }

                // Simulate async action completion after 2 seconds
                if let Some(started) = self.action1_started
                    && self.elapsed.saturating_sub(started) >= Duration::from_secs(2)
                {
                    self.action1_state = ActionState::Succeeded;
                    self.action1_started = None;
                }
            }
            Message::ToggleGyro(v) => self.gyro_on = v,
            Message::ToggleComms(v) => self.comms_on = v,
            Message::ToggleLights(v) => self.lights_on = v,
            Message::ToggleRetro(v) => self.retro_on = v,
            Message::RetroGuard(v) => self.retro_guarded = v,
            Message::AbortGuard(state) => self.abort_state = state,
            Message::AbortExecute => {
                self.abort_state = GuardState::Armed;
            }
            Message::FlightMode(idx) => self.flight_mode = idx,
            Message::AsyncAction1 => {
                self.action1_state = ActionState::Pending;
                self.action1_started = Some(self.elapsed);
            }
            Message::AsyncAction1Complete => {
                self.action1_state = ActionState::Succeeded;
                self.action1_started = None;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let colors = MercuryColors::default();

        // ── Header ──────────────────────────────────────────────
        let title = text("QUECKSILBER CONTROL")
            .size(18)
            .color(colors.text);

        let header = container(title);

        // ── Left column: MET timer + indicator panels ───────────
        let timer: Element<'_, ()> = canvas(
            EventTimer::new("MET", self.elapsed)
                .running(true)
                .colors(colors),
        )
        .width(140)
        .height(140)
        .into();

        let timer_container = container(timer.map(|_: ()| unreachable!()))
            .center_x(Length::Fill);

        let systems_panel = indicator_panel("SYSTEMS", &self.systems, 4);
        let abort_panel = indicator_panel("ABORT SEQUENCE", &self.abort_sequence, 4);

        let left_col = column![timer_container, systems_panel, abort_panel]
            .spacing(12)
            .width(300);

        // ── Center: attitude indicator + instrument gauges ─────────
        let adi: Element<'_, ()> = canvas(
            AttitudeIndicator::new(self.roll, self.pitch, self.yaw)
                .roll_range(-30.0, 30.0)
                .pitch_range(-20.0, 20.0)
                .yaw_range(-30.0, 30.0)
                .colors(colors),
        )
        .width(170)
        .height(170)
        .into();

        let adi_container = container(adi.map(|_: ()| unreachable!()))
            .center_x(Length::Fill);

        let gauge_size = 170;

        let cabin_gauge: Element<'_, ()> = canvas(
            Gauge::new("CABIN PRESS", self.cabin_pressure, 0.0, 20.0)
                .unit("psi")
                .arc(0.0, 10.0, IndicatorStatus::Caution)
                .arc(10.0, 16.0, IndicatorStatus::Nominal)
                .arc(16.0, 18.0, IndicatorStatus::Caution)
                .arc(18.0, 20.0, IndicatorStatus::Alert)
                .major_ticks(4)
                .minor_ticks(4),
        )
        .width(gauge_size)
        .height(gauge_size)
        .into();

        let o2_gauge: Element<'_, ()> = canvas(
            Gauge::new("O2 SUPPLY", self.o2_supply, 0.0, 100.0)
                .unit("%")
                .arc(0.0, 20.0, IndicatorStatus::Alert)
                .arc(20.0, 40.0, IndicatorStatus::Caution)
                .arc(40.0, 100.0, IndicatorStatus::Nominal)
                .major_ticks(5)
                .minor_ticks(4),
        )
        .width(gauge_size)
        .height(gauge_size)
        .into();

        let volts_gauge: Element<'_, ()> = canvas(
            Gauge::new("DC VOLTS", self.dc_volts, 20.0, 32.0)
                .unit("V")
                .arc(20.0, 24.0, IndicatorStatus::Alert)
                .arc(24.0, 30.0, IndicatorStatus::Nominal)
                .arc(30.0, 32.0, IndicatorStatus::Caution)
                .major_ticks(6)
                .minor_ticks(1),
        )
        .width(gauge_size)
        .height(gauge_size)
        .into();

        let attitude_gauge: Element<'_, ()> = canvas(
            Gauge::new("ATT RATE", self.attitude_rate, -5.0, 5.0)
                .unit("°/s")
                .arc(-5.0, -2.0, IndicatorStatus::Alert)
                .arc(-2.0, 2.0, IndicatorStatus::Nominal)
                .arc(2.0, 5.0, IndicatorStatus::Alert)
                .major_ticks(5)
                .minor_ticks(1),
        )
        .width(gauge_size)
        .height(gauge_size)
        .into();

        let gauge_row_1 = row![
            cabin_gauge.map(|_: ()| unreachable!()),
            o2_gauge.map(|_: ()| unreachable!()),
        ]
        .spacing(8);

        let gauge_row_2 = row![
            volts_gauge.map(|_: ()| unreachable!()),
            attitude_gauge.map(|_: ()| unreachable!()),
        ]
        .spacing(8);

        let gauge_grid = container(
            column![gauge_row_1, gauge_row_2]
                .spacing(8)
                .align_x(iced::Alignment::Center),
        )
        .center_x(Length::Fill);

        let center_col = column![adi_container, gauge_grid]
            .spacing(8)
            .width(Length::Fill)
            .align_x(iced::Alignment::Center);

        // ── Right column: switches, abort, flight mode dial ─────
        let gyro_toggle: Element<'_, Message> =
            toggle_switch("GYRO", self.gyro_on, Message::ToggleGyro).into();
        let comms_toggle: Element<'_, Message> =
            toggle_switch("COMMS", self.comms_on, Message::ToggleComms).into();
        let lights_toggle: Element<'_, Message> =
            toggle_switch("LIGHTS", self.lights_on, Message::ToggleLights).into();
        let retro_toggle: Element<'_, Message> =
            toggle_switch("RETRO SEQ", self.retro_on, Message::ToggleRetro)
                .guarded(self.retro_guarded, Message::RetroGuard)
                .into();

        let abort_btn = guarded_button(
            "ABORT",
            self.abort_state,
            Message::AbortGuard(GuardState::Confirming),
            Message::AbortExecute,
            Message::AbortGuard(GuardState::Armed),
        );

        let flight_label = text("FLIGHT MODE")
            .size(10)
            .color(colors.tick_mark);

        let flight_dial: Element<'_, Message> = canvas(SelectorDial::new(
            &["AUTO", "FBW", "MAN", "REENTRY"],
            self.flight_mode,
            Message::FlightMode,
        ))
        .width(150)
        .height(150)
        .into();

        let dial_section = column![flight_label, flight_dial]
            .spacing(4)
            .align_x(iced::Alignment::Center);

        let right_col = column![
            gyro_toggle,
            comms_toggle,
            lights_toggle,
            retro_toggle,
            abort_btn,
            Space::new().height(8),
            dial_section,
        ]
        .spacing(6)
        .width(210);

        // ── Bottom bar: status bars + async action button ───────
        let fuel_bar: Element<'_, ()> = canvas(
            StatusBar::new("FUEL", self.fuel_remaining, 0.0, 100.0)
                .unit("%")
                .zone(0.0, 20.0, IndicatorStatus::Alert)
                .zone(20.0, 40.0, IndicatorStatus::Caution)
                .zone(40.0, 100.0, IndicatorStatus::Nominal),
        )
        .width(Length::Fill)
        .height(40)
        .into();

        let battery_bar: Element<'_, ()> = canvas(
            StatusBar::new("BATTERY", self.battery, 0.0, 100.0)
                .unit("%")
                .zone(0.0, 15.0, IndicatorStatus::Alert)
                .zone(15.0, 30.0, IndicatorStatus::Caution)
                .zone(30.0, 100.0, IndicatorStatus::Nominal),
        )
        .width(Length::Fill)
        .height(40)
        .into();

        let signal_bar: Element<'_, ()> = canvas(
            StatusBar::new("SIGNAL", self.signal, 0.0, 100.0)
                .unit("dBm")
                .zone(0.0, 30.0, IndicatorStatus::Alert)
                .zone(30.0, 60.0, IndicatorStatus::Caution)
                .zone(60.0, 100.0, IndicatorStatus::Nominal),
        )
        .width(Length::Fill)
        .height(40)
        .into();

        let action_btn =
            async_action_button("TELEMETRY SYNC", self.action1_state, Message::AsyncAction1);

        let bottom_bar = row![
            fuel_bar.map(|_: ()| unreachable!()),
            battery_bar.map(|_: ()| unreachable!()),
            signal_bar.map(|_: ()| unreachable!()),
            container(action_btn).width(200),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        // ── Assemble ────────────────────────────────────────────
        let main_area = row![left_col, center_col, right_col]
            .spacing(16)
            .height(Length::Fill);

        let content = column![header, main_area, bottom_bar]
            .spacing(12)
            .padding(16);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
