use std::f64::consts::PI;

use aksel::{Float, PlotPoint, PlotRect, scale::Linear};
use iced::{Color, Element, Task, Theme};
use iced_aksel::{Axis, Chart, DragDelta, Shape, State, axis, plot::Items, shape::Polygon};

const X_ID: &str = "linear_x";
const Y_ID: &str = "linear_y";

type AxisId = &'static str;

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application State ---

struct ExampleApp {
    state: State<AxisId, f64>,

    star_system: SolarSystem,
}

#[derive(Debug, Clone)]
enum Message {
    ChartDragged(DragDelta),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut chart_state = State::new();

        // Initialize axes 0-100 on both axis
        chart_state.set_axis(
            X_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom),
        );
        chart_state.set_axis(
            Y_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Right),
        );
        (
            Self {
                state: chart_state,
                star_system: SolarSystem::new(vec![], 5.0),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
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
    fn new(points: Vec<PlotPoint>, star_size: f64) -> Self {
        SolarSystem { points, star_size }
    }
}

impl Items<f64> for SolarSystem {
    fn draw(&self, plot: &mut iced_aksel::Plot<f64, iced::Renderer>, theme: &iced::Theme) {
        // Prepare values
        let bounds = plot.bounds();
        let filtered_star_points: Vec<&PlotPoint> =
            self.points.iter().filter(|&v| bounds.contains(v)).collect();

        for point in &self.points {
            // Create 2D rect for culling
            let rect = PlotRect::from_center(*point, self.star_size, self.star_size);

            // Cull data out of visible range
            if !bounds.intersects(&rect) {
                continue;
            }

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
        let x = point.x + size * angle.cos() * angle_offset;
        let y = point.y + size * angle.sin() * angle_offset;

        points.push(PlotPoint::new(x, y));

        angle_offset += angle;
    }

    points
}
