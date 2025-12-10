#![allow(unused)]

use std::f64::consts::PI;

use aksel::{PlotPoint, scale::Linear};
use iced::{Color, Element, Task, Theme};
use iced_aksel::{Axis, Chart, DragDelta, State, axis, plot::Items, shape::Polygon};
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
    star_system: SolarSystem,
}

#[derive(Debug, Clone)]
enum Message {
    ChartDragged(DragDelta),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut chart_state = State::new()
            .with_axis(
                "x_axis_id",
                Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom),
            )
            .with_axis(
                "y_axis_id",
                Axis::new(Linear::new(0.0, 100.0), axis::Position::Right),
            );

        // Initialize axes 0-100 on both axis
        chart_state.set_axis(
            X_ID,
            Axis::new(Linear::new(AXIS_MIN, AXIS_MAX), axis::Position::Bottom),
        );
        chart_state.set_axis(
            Y_ID,
            Axis::new(Linear::new(AXIS_MIN, AXIS_MAX), axis::Position::Right),
        );
        (
            Self {
                chart_state,
                star_system: SolarSystem::new(generate_values(100), 5.0),
            },
            Task::none(),
        )
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .layer(&self.star_system, X_ID, Y_ID)
            .on_drag(Message::ChartDragged);
        chart.into()
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Theme::Dark)
            .run()
    }
}

struct SolarSystem {
    points: Vec<PlotPoint>,
    star_size: f64,
}

impl SolarSystem {
    const fn new(points: Vec<PlotPoint>, star_size: f64) -> Self {
        Self { points, star_size }
    }
}

impl Items<f64> for SolarSystem {
    fn draw(&self, plot: &mut iced_aksel::Plot<f64, iced::Renderer>, _theme: &iced::Theme) {
        // Prepare values
        let bounds = plot.bounds();

        for point in &self.points {
            // Create 2D rect for culling
            // let rect = PlotRect::from_center(*point, self.star_size, self.star_size);

            // Cull data out of visible range
            // if !bounds.intersects(&rect) {
            //     continue;
            // }

            // Create shape
            let geometry = star_geometry(point, self.star_size);
            let shape = Polygon::new(geometry).fill(Color::WHITE);

            plot.add_shape(shape);
        }
    }
}

/// Creates a 5 pointed star from this center with size
fn star_geometry(point: &PlotPoint, size: f64) -> Vec<PlotPoint> {
    let mut points = Vec::new();
    let angle = 2.0 * PI / 5.0;
    let mut angle_offset = 0.0;

    for _ in 0..5 {
        let x = (size * angle.cos()).mul_add(angle_offset, point.x);
        let y = (size * angle.sin()).mul_add(angle_offset, point.y);

        points.push(PlotPoint::new(x, y));

        angle_offset += angle;
    }

    points
}

fn generate_values(amount: usize) -> Vec<PlotPoint> {
    let mut rng = rand::rng();
    (0..amount)
        .map(|_| PlotPoint {
            x: rng.random_range(AXIS_MIN..=AXIS_MAX),
            y: rng.random_range(AXIS_MIN..=AXIS_MAX),
        })
        .collect()
}
