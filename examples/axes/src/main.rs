#![allow(unused)]

use std::f64::consts::PI;

use aksel::{PlotPoint, scale::Linear};
use iced::{Color, Element, Task, Theme};
use iced_aksel::{
    Axis, Chart, DragDelta, State,
    axis::{self, GridLine, Label},
    plot::PlotData,
    shape::Polygon,
};
use rand::Rng;

const X_ID: &str = "linear_x";
const Y_ID: &str = "linear_y";

const AXIS_MIN: f64 = 0.0;
const AXIS_MAX: f64 = 100.0;

type AxisId = &'static str;

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application State ---
struct ExampleApp {
    chart_state: State<AxisId, f64>,
}

#[derive(Debug, Clone)]
enum Message {}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut chart_state = State::new();

        // TODO: Add runtime grid on/off for axis
        // Initialize axes 0-100 on both axis
        let x_axis = Axis::new(Linear::new(AXIS_MIN, AXIS_MAX), axis::Position::Bottom)
            .with_grid_renderer(|ctx| {
                // Very simple gridline implementation currently.
                let major_gridline = GridLine {
                    thickness: 1.into(),
                };
                let minor_gridline = GridLine {
                    thickness: 0.5.into(),
                };
                match ctx.level {
                    0 => Some(major_gridline),
                    _ => Some(minor_gridline),
                }
            })
            .with_custom_label_policy(|ctx| {
                // Here you can use the context that you are given to determine if a specific tick should be rendered
                // This feature is for those the want ultimate control. Its not necessary for most usecases
                //
                // Example of making different decisions:
                let decision = axis::LabelDecision::Render;
                decision
            })
            .with_cursor_formatter(|ctx| {
                let cursor_label = iced_aksel::axis::Label {
                    size: 32.into(),
                    content: "Test".to_string(),
                };
                Some(cursor_label)
            })
            .with_tick_renderer(|ctx| {
                // Here you get all the info you would need to determine which labels should show
                // And which should show in different ways.
                let level = ctx.tick.level;

                let major_label = iced_aksel::axis::Label {
                    size: 10.into(),
                    content: "Major".to_string(),
                };
                let minor_label = iced_aksel::axis::Label {
                    size: 6.into(),
                    content: "Minor".to_string(),
                };

                let tick_line_major = iced_aksel::axis::TickLine {
                    label: Some(major_label),
                    length: 4.into(),
                    thickness: 1.into(),
                };
                let tick_line_minor = iced_aksel::axis::TickLine {
                    label: Some(minor_label),
                    length: 2.into(),
                    thickness: 0.5.into(),
                };

                match level {
                    0 => Some(tick_line_major),
                    1 => Some(tick_line_minor),
                    _ => None,
                }
            })
            .with_thickness(32.)
            .with_label_spacing(0.0)
            .skip_overlapping_labels(6.);
        let y_axis = Axis::new(Linear::new(AXIS_MIN, AXIS_MAX), axis::Position::Right);

        chart_state.set_axis(X_ID, x_axis);
        chart_state.set_axis(Y_ID, y_axis);

        (Self { chart_state }, Task::none())
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state);
        chart.into()
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Theme::Dark)
            .run()
    }
}
