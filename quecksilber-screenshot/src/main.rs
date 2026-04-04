use clap::Parser;
use iced::widget::canvas::{self, Path};
use iced::{Element, Font, Length, Point, Rectangle, Renderer, Size, Task, Theme, mouse, window};
use quecksilber::widgets::{
    ArmStyle, DualGauge, Gauge, HorizontalGauge, LeverOrientation, LeverSwitch, Origin,
    RotarySelector, Subdivision,
};
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Debug, Parser)]
struct Args {
    /// Only regenerate the screenshot with this name (don't clear the directory).
    name: Option<String>,
}

static ARGS: OnceLock<Args> = OnceLock::new();

const B612: Font = Font::with_name("B612");

struct Variant {
    name: &'static str,
    origin: Origin,
    subdivision: Subdivision,
    mid_ticks: bool,
    arm_style: ArmStyle,
    range: (f32, f32),
    label_every: u32,
    gap: f32,
}

const DEFAULT: Variant = Variant {
    name: "",
    origin: Origin::Bottom,
    arm_style: ArmStyle::Blunt,
    mid_ticks: false,
    subdivision: Subdivision::Integer,
    range: (0.0, 15.0),
    label_every: 3,
    gap: 0.3,
};

fn variants() -> Vec<Variant> {
    vec![
        Variant {
            name: "sub_none",
            subdivision: Subdivision::None,
            ..DEFAULT
        },
        Variant {
            name: "sub_integer",
            ..DEFAULT
        },
        Variant {
            name: "sub_mid_only",
            subdivision: Subdivision::None,
            mid_ticks: true,
            ..DEFAULT
        },
        Variant {
            name: "sub_integer_mid",
            mid_ticks: true,
            ..DEFAULT
        },
        Variant {
            name: "origin_centered",
            origin: Origin::Centered,
            ..DEFAULT
        },
        Variant {
            name: "origin_left",
            origin: Origin::Left,
            ..DEFAULT
        },
        Variant {
            name: "origin_right",
            origin: Origin::Right,
            ..DEFAULT
        },
        Variant {
            name: "centered_100_mid",
            origin: Origin::Centered,
            mid_ticks: true,
            range: (0.0, 100.0),
            label_every: 10,
            gap: 0.2,
            ..DEFAULT
        },
        Variant {
            name: "mid_even",
            mid_ticks: true,
            range: (0.0, 20.0),
            label_every: 4,
            ..DEFAULT
        },
        Variant {
            name: "integer_mid_even",
            mid_ticks: true,
            range: (0.0, 20.0),
            label_every: 4,
            ..DEFAULT
        },
        Variant {
            name: "arm_needle",
            arm_style: ArmStyle::Needle,
            ..DEFAULT
        },
        Variant {
            name: "arm_needle_centered",
            arm_style: ArmStyle::Needle,
            origin: Origin::Centered,
            ..DEFAULT
        },
        Variant {
            name: "arm_needle_100",
            arm_style: ArmStyle::Needle,
            origin: Origin::Centered,
            range: (0.0, 100.0),
            label_every: 10,
            gap: 0.2,
            mid_ticks: true,
            ..DEFAULT
        },
        Variant {
            name: "fraction_4",
            subdivision: Subdivision::Fraction(4),
            range: (0.0, 4.0),
            label_every: 1,
            ..DEFAULT
        },
        Variant {
            name: "every_5",
            subdivision: Subdivision::Every(5),
            range: (0.0, 100.0),
            label_every: 20,
            ..DEFAULT
        },
    ]
}

fn main() -> iced::Result {
    ARGS.set(Args::parse()).expect("args already set");
    iced::application(boot, update, view)
        .title("Quecksilber Screenshot")
        .theme(quecksilber::themes::cockpit())
        .window_size(Size::new(200.0, 200.0))
        .font(include_bytes!("../../quecksilber/assets/B612-Regular.ttf").as_slice())
        .run()
}

enum Screenshot {
    GaugeVariant(usize),
    DualGauge,
    HorizontalGauge,
    RotarySelector,
    LeverSwitch3Pos0,
    LeverSwitch3Pos1,
    LeverSwitch3Pos2,
    LeverSwitch2,
    LeverSwitchVertical,
}

fn screenshot_list() -> Vec<(&'static str, Screenshot)> {
    let mut list: Vec<(&str, Screenshot)> = variants()
        .iter()
        .enumerate()
        .map(|(i, v)| (v.name, Screenshot::GaugeVariant(i)))
        .collect();
    list.push(("dual_gauge", Screenshot::DualGauge));
    list.push(("horizontal_gauge", Screenshot::HorizontalGauge));
    list.push(("rotary_selector", Screenshot::RotarySelector));
    list.push(("lever_switch_3_pos0", Screenshot::LeverSwitch3Pos0));
    list.push(("lever_switch_3_pos1", Screenshot::LeverSwitch3Pos1));
    list.push(("lever_switch_3_pos2", Screenshot::LeverSwitch3Pos2));
    list.push(("lever_switch_2", Screenshot::LeverSwitch2));
    list.push(("lever_switch_vertical", Screenshot::LeverSwitchVertical));

    if let Some(filter) = &ARGS.get().expect("args not set").name {
        list.retain(|(name, _)| *name == filter.as_str());
        assert!(
            !list.is_empty(),
            "no screenshot named '{filter}'. available: {}",
            screenshot_names().join(", ")
        );
    }

    list
}

fn screenshot_names() -> Vec<String> {
    let mut names: Vec<String> = variants().iter().map(|v| v.name.to_string()).collect();
    names.push("dual_gauge".to_string());
    names.push("horizontal_gauge".to_string());
    names.push("rotary_selector".to_string());
    names.push("lever_switch_3_pos0".to_string());
    names.push("lever_switch_3_pos1".to_string());
    names.push("lever_switch_3_pos2".to_string());
    names.push("lever_switch_2".to_string());
    names.push("lever_switch_vertical".to_string());
    names
}

fn boot() -> (State, Task<Message>) {
    let vars = variants();
    let screenshots = screenshot_list();
    let widget = make_widget(&vars, &screenshots[0].1);
    let state = State {
        current: 0,
        variants: vars,
        screenshots,
        widget,
    };
    let clear = ARGS.get().expect("args not set").name.is_none();
    let task = Task::perform(
        async move {
            if clear {
                let _ = std::fs::remove_dir_all("screenshots");
            }
            std::fs::create_dir_all("screenshots").expect("failed to create screenshots dir");
        },
        |()| Message::Capture,
    );
    (state, task)
}

enum Widget {
    Gauge(Gauge),
    DualGauge(DualGauge),
    HorizontalGauge(HorizontalGauge),
    RotarySelector {
        selected: usize,
    },
    LeverSwitch {
        positions: usize,
        selected: usize,
        orientation: LeverOrientation,
        labels: Vec<&'static str>,
        title: &'static str,
    },
}

fn make_gauge(v: &Variant) -> Gauge {
    Gauge::new(v.range.0, v.range.1)
        .label_every(v.label_every)
        .label("CABIN\nPRESSURE")
        .gap(v.gap)
        .font(B612)
        .origin(v.origin)
        .subdivision(v.subdivision)
        .mid_ticks(v.mid_ticks)
        .arm_style(v.arm_style)
}

fn make_widget(variants: &[Variant], screenshot: &Screenshot) -> Widget {
    match screenshot {
        Screenshot::GaugeVariant(i) => Widget::Gauge(make_gauge(&variants[*i])),
        Screenshot::RotarySelector => Widget::RotarySelector { selected: 2 },
        Screenshot::HorizontalGauge => Widget::HorizontalGauge(
            HorizontalGauge::new(0.0, 100.0)
                .label_every(20)
                .label("TEMP")
                .value(50.0)
                .font(B612),
        ),
        Screenshot::DualGauge => Widget::DualGauge(
            DualGauge::new()
                .top_label("TOP")
                .right_label("RIGHT")
                .bottom_label("BOTTOM")
                .left_label("LEFT")
                .left_range(0.0, 100.0)
                .left_label_every(50)
                .right_range(0.0, 100.0)
                .right_label_every(20)
                .left_value(65.0)
                .right_value(30.0)
                .font(B612),
        ),
        Screenshot::LeverSwitch3Pos0 => Widget::LeverSwitch {
            positions: 3,
            selected: 0,
            orientation: LeverOrientation::Horizontal,
            labels: vec!["AUTO", "OFF", "MAN"],
            title: "STDBY BAT",
        },
        Screenshot::LeverSwitch3Pos1 => Widget::LeverSwitch {
            positions: 3,
            selected: 1,
            orientation: LeverOrientation::Horizontal,
            labels: vec!["AUTO", "OFF", "MAN"],
            title: "STDBY BAT",
        },
        Screenshot::LeverSwitch3Pos2 => Widget::LeverSwitch {
            positions: 3,
            selected: 2,
            orientation: LeverOrientation::Horizontal,
            labels: vec!["AUTO", "OFF", "MAN"],
            title: "STDBY BAT",
        },
        Screenshot::LeverSwitch2 => Widget::LeverSwitch {
            positions: 2,
            selected: 0,
            orientation: LeverOrientation::Horizontal,
            labels: vec!["NORM", "OFF"],
            title: "",
        },
        Screenshot::LeverSwitchVertical => Widget::LeverSwitch {
            positions: 2,
            selected: 1,
            orientation: LeverOrientation::Vertical,
            labels: vec!["READY", "OFF"],
            title: "",
        },
    }
}

struct State {
    current: usize,
    variants: Vec<Variant>,
    screenshots: Vec<(&'static str, Screenshot)>,
    widget: Widget,
}

#[derive(Debug, Clone)]
enum Message {
    Capture,
    Screenshot(window::Screenshot),
    Saved,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Capture => window::latest()
            .and_then(window::screenshot)
            .map(Message::Screenshot),
        Message::Screenshot(screenshot) => {
            let size = screenshot.size;
            let rgba = screenshot.rgba.to_vec();
            let (name, _) = &state.screenshots[state.current];
            let path = format!("screenshots/{name}.png");

            Task::perform(
                async move {
                    std::fs::create_dir_all("screenshots")
                        .expect("failed to create screenshots dir");
                    image::save_buffer(
                        &path,
                        &rgba,
                        size.width,
                        size.height,
                        image::ColorType::Rgba8,
                    )
                    .expect("failed to save screenshot");
                },
                |()| Message::Saved,
            )
        }
        Message::Saved => {
            state.current += 1;
            if state.current < state.screenshots.len() {
                state.widget = make_widget(&state.variants, &state.screenshots[state.current].1);
                Task::perform(
                    async { std::thread::sleep(Duration::from_millis(200)) },
                    |()| Message::Capture,
                )
            } else {
                iced::exit()
            }
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    iced::widget::canvas(WidgetView {
        widget: &state.widget,
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

struct WidgetView<'a> {
    widget: &'a Widget,
}

impl<'a> canvas::Program<Message> for WidgetView<'a> {
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

        let bg = Path::rectangle(iced::Point::ORIGIN, bounds.size());
        frame.fill(&bg, theme.palette().background);

        let full_radius = bounds.width.min(bounds.height) / 2.0 * 0.85;
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);

        match self.widget {
            Widget::Gauge(gauge) => gauge.draw_at(&mut frame, theme, center, full_radius),
            Widget::DualGauge(dg) => dg.draw_at(&mut frame, theme, center, full_radius),
            Widget::HorizontalGauge(sg) => sg.draw_at(&mut frame, theme, center, full_radius),
            Widget::RotarySelector { selected } => {
                let rs = RotarySelector::new(vec!["OFF", "1", "2", "3", "BOTH"], *selected, |_| {})
                    .left_label("L")
                    .right_label("R")
                    .font(B612);
                rs.draw_at(&mut frame, theme, center, full_radius);
            }
            Widget::LeverSwitch {
                positions,
                selected,
                orientation,
                labels,
                title,
            } => {
                let ls = LeverSwitch::new(*positions, *selected, |_| {})
                    .orientation(*orientation)
                    .labels(labels.clone())
                    .title(*title)
                    .font(B612);
                ls.draw_at(&mut frame, theme, center, full_radius);
            }
        }

        vec![frame.into_geometry()]
    }
}
