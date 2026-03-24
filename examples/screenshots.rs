//! Generates screenshots of every quecksilber widget.
//!
//! Run with: `cargo run --example screenshots`
//!
//! Saves PNGs to `./screenshots/`. Each widget is rendered one at a time
//! in a single window that resizes between captures, then the app exits.

use std::path::PathBuf;
use std::time::Duration;

use iced::widget::{canvas, column, container, row, text};
use iced::window;
use iced::{Element, Length, Subscription, Task};

use quecksilber::{
    async_action_button, guarded_button, indicator_panel, mercury_theme, toggle_switch,
    ActionState, EventTimer, Gauge, GuardState, Indicator, IndicatorStatus, MercuryColors,
    SelectorDial, StatusBar,
};

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title("Quecksilber Screenshots")
        .theme(mercury_theme())
        .window_size((500.0, 300.0))
        .subscription(App::subscription)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ScreenshotTaken(window::Screenshot),
}

struct App {
    /// Index into WIDGETS of the widget currently being rendered.
    current: usize,
    /// Frame counter since last widget switch — wait a few frames before capture.
    frames: u32,
    /// Whether we're waiting for a screenshot result.
    capturing: bool,
}

/// Descriptor for one screenshot target.
struct Widget {
    name: &'static str,
    build: fn() -> Element<'static, Message>,
}

const SETTLE_FRAMES: u32 = 8;

fn widgets() -> Vec<Widget> {
    vec![
        Widget { name: "indicator", build: build_indicator },
        Widget { name: "indicator_panel", build: build_indicator_panel },
        Widget { name: "gauge", build: build_gauge },
        Widget { name: "status_bar", build: build_status_bar },
        Widget { name: "event_timer", build: build_event_timer },
        Widget { name: "toggle_switch", build: build_toggle_switch },
        Widget { name: "guarded_button", build: build_guarded_button },
        Widget { name: "selector_dial", build: build_selector_dial },
        Widget { name: "async_action_button", build: build_async_action_button },
    ]
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        std::fs::create_dir_all("./screenshots").ok();
        (
            Self {
                current: 0,
                frames: 0,
                capturing: false,
            },
            Task::none(),
        )
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.current < widgets().len() {
            iced::time::every(Duration::from_millis(50)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick => {
                if self.capturing || self.current >= widgets().len() {
                    return Task::none();
                }

                self.frames += 1;

                if self.frames >= SETTLE_FRAMES {
                    self.capturing = true;
                    return window::oldest().then(|id| match id {
                        Some(id) => window::screenshot(id).map(Message::ScreenshotTaken),
                        None => Task::none(),
                    });
                }

                Task::none()
            }
            Message::ScreenshotTaken(screenshot) => {
                let all = widgets();
                if self.current < all.len() {
                    let name = all[self.current].name;
                    let path = PathBuf::from(format!("./screenshots/{name}.png"));
                    save_screenshot(&screenshot, &path);
                    eprintln!("[{}/{}] Saved {}", self.current + 1, all.len(), path.display());
                }

                self.current += 1;
                self.frames = 0;
                self.capturing = false;

                if self.current >= all.len() {
                    eprintln!("All screenshots saved.");
                    return window::oldest().then(|id| match id {
                        Some(id) => window::close(id),
                        None => Task::none(),
                    });
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let all = widgets();

        let widget = if self.current < all.len() {
            (all[self.current].build)()
        } else {
            text("Done.").size(16).color(MercuryColors::default().text).into()
        };

        container(widget)
            .width(Length::Fill)
            .height(Length::Fill)
            .center(Length::Fill)
            .padding(20)
            .into()
    }
}

// ── Widget builders ─────────────────────────────────────────────────

fn build_indicator() -> Element<'static, Message> {
    let colors = MercuryColors::default();
    let indicators: Vec<Element<'static, Message>> = [
        ("OFF", IndicatorStatus::Off),
        ("NOMINAL", IndicatorStatus::Nominal),
        ("CAUTION", IndicatorStatus::Caution),
        ("ALERT", IndicatorStatus::Alert),
    ]
    .into_iter()
    .map(|(label, status)| {
        let el: Element<'static, ()> = canvas(Indicator::new(label, status).colors(colors))
            .width(72)
            .height(90)
            .into();
        el.map(|_: ()| unreachable!())
    })
    .collect();

    row(indicators).spacing(8).into()
}

fn build_indicator_panel() -> Element<'static, Message> {
    let indicators = vec![
        ("ELEC", IndicatorStatus::Nominal),
        ("COMMS", IndicatorStatus::Nominal),
        ("O2", IndicatorStatus::Caution),
        ("CABIN", IndicatorStatus::Nominal),
        ("RETRO", IndicatorStatus::Off),
        ("GYRO", IndicatorStatus::Alert),
        ("ENVIRO", IndicatorStatus::Nominal),
        ("STAB", IndicatorStatus::Nominal),
    ];
    indicator_panel("SYSTEMS", &indicators, 4)
}

fn build_gauge() -> Element<'static, Message> {
    let gauge = Gauge::new("CABIN PRESS", 14.2, 0.0, 20.0)
        .unit("psi")
        .arc(0.0, 10.0, IndicatorStatus::Caution)
        .arc(10.0, 16.0, IndicatorStatus::Nominal)
        .arc(16.0, 18.0, IndicatorStatus::Caution)
        .arc(18.0, 20.0, IndicatorStatus::Alert)
        .major_ticks(4)
        .minor_ticks(4);

    let el: Element<'static, ()> = canvas(gauge).width(200).height(200).into();
    el.map(|_: ()| unreachable!())
}

fn build_status_bar() -> Element<'static, Message> {
    let bars: Vec<Element<'static, Message>> = [
        StatusBar::new("FUEL", 72.0, 0.0, 100.0)
            .unit("%")
            .zone(0.0, 20.0, IndicatorStatus::Alert)
            .zone(20.0, 40.0, IndicatorStatus::Caution)
            .zone(40.0, 100.0, IndicatorStatus::Nominal),
        StatusBar::new("BATTERY", 91.0, 0.0, 100.0)
            .unit("%")
            .zone(0.0, 15.0, IndicatorStatus::Alert)
            .zone(15.0, 30.0, IndicatorStatus::Caution)
            .zone(30.0, 100.0, IndicatorStatus::Nominal),
        StatusBar::new("SIGNAL", 45.0, 0.0, 100.0)
            .unit("dBm")
            .zone(0.0, 30.0, IndicatorStatus::Alert)
            .zone(30.0, 60.0, IndicatorStatus::Caution)
            .zone(60.0, 100.0, IndicatorStatus::Nominal),
    ]
    .into_iter()
    .map(|bar| {
        let el: Element<'static, ()> = canvas(bar).width(Length::Fill).height(40).into();
        el.map(|_: ()| unreachable!())
    })
    .collect();

    column(bars).spacing(8).into()
}

fn build_event_timer() -> Element<'static, Message> {
    let timer = EventTimer::new("MET", Duration::from_secs(3723)).running(true);

    let el: Element<'static, ()> = canvas(timer).width(180).height(180).into();
    el.map(|_: ()| unreachable!())
}

fn build_toggle_switch() -> Element<'static, Message> {
    let on: Element<'static, Message> =
        toggle_switch("GYRO POWER", true, |_| Message::Tick).into();
    let off: Element<'static, Message> =
        toggle_switch("CABIN LIGHTS", false, |_| Message::Tick).into();
    let guarded: Element<'static, Message> =
        toggle_switch("RETRO SEQ", false, |_| Message::Tick)
            .guarded(true, |_| Message::Tick)
            .into();

    column![on, off, guarded].spacing(8).into()
}

fn build_guarded_button() -> Element<'static, Message> {
    let armed = guarded_button(
        "ABORT",
        GuardState::Armed,
        Message::Tick,
        Message::Tick,
        Message::Tick,
    );
    let confirming = guarded_button(
        "ABORT",
        GuardState::Confirming,
        Message::Tick,
        Message::Tick,
        Message::Tick,
    );

    column![armed, confirming].spacing(12).into()
}

fn build_selector_dial() -> Element<'static, Message> {
    let dial = SelectorDial::new(
        &["AUTO", "FBW", "MAN", "REENTRY"],
        1,
        |_| Message::Tick,
    );

    canvas(dial).width(180).height(180).into()
}

fn build_async_action_button() -> Element<'static, Message> {
    let idle = async_action_button("TELEMETRY SYNC", ActionState::Idle, Message::Tick);
    let pending = async_action_button("UPLOADING", ActionState::Pending, Message::Tick);
    let succeeded = async_action_button("DATA SENT", ActionState::Succeeded, Message::Tick);
    let failed = async_action_button("TX FAILED", ActionState::Failed, Message::Tick);

    row![idle, pending, succeeded, failed].spacing(8).into()
}

// ── Screenshot saving ───────────────────────────────────────────────

fn save_screenshot(screenshot: &window::Screenshot, path: &PathBuf) {
    let size = screenshot.size;
    if let Some(img) =
        image::RgbaImage::from_raw(size.width, size.height, screenshot.rgba.to_vec())
    {
        if let Err(e) = img.save(path) {
            eprintln!("Failed to save screenshot: {e}");
        }
    } else {
        eprintln!("Failed to create image buffer from screenshot data");
    }
}
