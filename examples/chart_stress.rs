//! Chart Stress Test
//!
//! A stress-testing example for the chart widget with:
//! - FPS counter to monitor performance
//! - GUI-configurable stress parameters
//! - Multiple shape types (rectangles and polylines)
//! - Real-time adjustable rendering complexity

use aksel::{PlotPoint, scale::Linear};
use iced::{
    Color, Element, Subscription, Task, Theme,
    widget::{Slider, button, column, row, text},
};
use iced_aksel::{Axis, Chart, Length, Plot, State, Stroke, axis::Position, plot, shape};
use rand::Rng;
use std::time::Instant;

const AXIS_ID_X: &str = "x";
const AXIS_ID_Y: &str = "y";

type AxisId = &'static str;

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    RectangleCountChanged(f32),
    PolylineCountChanged(f32),
    PointsPerPolylineChanged(f32),
    Regenerate,
}

// Raw data structures - only store information needed to construct shapes
struct RectangleData {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    color: Color,
}

struct PolylineData {
    points: Vec<PlotPoint<f64>>,
    color: Color,
    thickness: f32,
}

// Data structures that hold shape information
struct StressRectangles {
    data: Vec<RectangleData>,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressRectangles {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        // Construct shapes from raw data during draw
        for rect_data in &self.data {
            let rect = shape::Rectangle::from_corners(
                PlotPoint::new(rect_data.x, rect_data.y),
                PlotPoint::new(
                    rect_data.x + rect_data.width,
                    rect_data.y + rect_data.height,
                ),
            )
            .fill(rect_data.color);

            plot.add_shape(rect);
        }
    }
}

struct StressPolylines {
    data: Vec<PolylineData>,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressPolylines {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        // Construct shapes from raw data during draw
        for line_data in &self.data {
            let stroke = Stroke::new(line_data.color, Length::Screen(line_data.thickness));
            let polyline = shape::Polyline::new(line_data.points.clone(), stroke);

            plot.add_shape(polyline);
        }
    }
}

struct StressTestApp {
    state: State<AxisId, f64>,
    rectangles_layer: StressRectangles,
    polylines_layer: StressPolylines,

    // Configuration
    rectangle_count: usize,
    polyline_count: usize,
    points_per_polyline: usize,

    // FPS counter
    last_frame_time: Option<Instant>,
    fps: f32,
    frame_times: Vec<f32>, // For calculating average frame time
}

impl StressTestApp {
    fn init() -> (Self, Task<Message>) {
        let mut state: State<AxisId, f64> = State::new();

        let axis_x = Axis::new(Linear::new(0.0, 1000.0), Position::Bottom);
        let axis_y = Axis::new(Linear::new(0.0, 1000.0), Position::Left);

        state.set_axis(AXIS_ID_X, axis_x);
        state.set_axis(AXIS_ID_Y, axis_y);

        let mut app = Self {
            state,
            rectangles_layer: StressRectangles { data: Vec::new() },
            polylines_layer: StressPolylines { data: Vec::new() },
            rectangle_count: 5000,
            polyline_count: 500,
            points_per_polyline: 100,
            last_frame_time: None,
            fps: 0.0,
            frame_times: Vec::with_capacity(60),
        };

        app.generate_shapes();

        (app, Task::none())
    }

    fn generate_shapes(&mut self) {
        // Clear existing data
        self.rectangles_layer.data.clear();
        self.polylines_layer.data.clear();

        let mut rng = rand::rng();

        // Generate rectangle data
        for _ in 0..self.rectangle_count {
            let x = rng.random_range(0.0..900.0);
            let y = rng.random_range(0.0..900.0);
            let width = rng.random_range(20.0..100.0);
            let height = rng.random_range(20.0..100.0);

            let color = Color::from_rgb(
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
            );

            self.rectangles_layer.data.push(RectangleData {
                x,
                y,
                width,
                height,
                color,
            });
        }

        // Generate polyline data
        for _ in 0..self.polyline_count {
            let mut points = Vec::with_capacity(self.points_per_polyline);

            // Start point
            let start_x = rng.random_range(0.0..200.0);
            let start_y = rng.random_range(0.0..1000.0);

            points.push(PlotPoint::new(start_x, start_y));

            // Generate connected points with some randomness
            let mut current_x: f64 = start_x;
            let mut current_y: f64 = start_y;

            for _ in 1..self.points_per_polyline {
                current_x += rng.random_range(5.0..20.0);
                current_y += rng.random_range(-50.0..50.0);
                current_y = current_y.clamp(0.0, 1000.0);

                if current_x > 1000.0 {
                    break;
                }

                points.push(PlotPoint::new(current_x, current_y));
            }

            let color = Color::from_rgba(
                rng.random_range(0.3..1.0),
                rng.random_range(0.3..1.0),
                rng.random_range(0.3..1.0),
                0.8,
            );

            let thickness = rng.random_range(1.0..3.0);

            self.polylines_layer.data.push(PolylineData {
                points,
                color,
                thickness,
            });
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                // Calculate FPS
                if let Some(last) = self.last_frame_time {
                    let delta = now.duration_since(last);
                    let delta_secs = delta.as_secs_f32();

                    if delta_secs > 0.0 {
                        // Instant FPS
                        let instant_fps = 1.0 / delta_secs;

                        // Smooth FPS with exponential moving average
                        self.fps = self.fps * 0.9 + instant_fps * 0.1;

                        // Track frame times for statistics
                        self.frame_times.push(delta_secs * 1000.0); // Convert to ms
                        if self.frame_times.len() > 60 {
                            self.frame_times.remove(0);
                        }
                    }
                }
                self.last_frame_time = Some(now);

                Task::none()
            }
            Message::RectangleCountChanged(value) => {
                self.rectangle_count = value as usize;
                self.generate_shapes();
                Task::none()
            }
            Message::PolylineCountChanged(value) => {
                self.polyline_count = value as usize;
                self.generate_shapes();
                Task::none()
            }
            Message::PointsPerPolylineChanged(value) => {
                self.points_per_polyline = value as usize;
                self.generate_shapes();
                Task::none()
            }
            Message::Regenerate => {
                self.generate_shapes();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .layer(&self.rectangles_layer, AXIS_ID_X, AXIS_ID_Y)
            .layer(&self.polylines_layer, AXIS_ID_X, AXIS_ID_Y);

        // Calculate statistics
        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        } else {
            0.0
        };

        let min_frame_time = self
            .frame_times
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let max_frame_time = self
            .frame_times
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let total_shapes = self.rectangle_count + self.polyline_count;
        let total_points =
            self.rectangle_count * 4 + self.polyline_count * self.points_per_polyline;

        // FPS and statistics display
        let fps_display = row![
            text(format!("FPS: {:.1}", self.fps)).size(20),
            text(format!(
                "  |  Frame Time: {:.2}ms (avg) {:.2}ms (min) {:.2}ms (max)",
                avg_frame_time, min_frame_time, max_frame_time
            ))
            .size(16),
        ]
        .spacing(10);

        let stats_display = row![
            text(format!(
                "Total Shapes: {} | Total Points: {}",
                total_shapes, total_points
            ))
            .size(16),
        ]
        .spacing(10);

        // Rectangle count slider
        let rectangle_slider = row![
            text(format!("Rectangles: {}", self.rectangle_count))
                .size(14)
                .width(150),
            Slider::new(
                0.0..=50000.0,
                self.rectangle_count as f32,
                Message::RectangleCountChanged
            )
            .step(500.0),
        ]
        .spacing(10)
        .padding(5);

        // Polyline count slider
        let polyline_slider = row![
            text(format!("Polylines: {}", self.polyline_count))
                .size(14)
                .width(150),
            Slider::new(
                0.0..=10000.0,
                self.polyline_count as f32,
                Message::PolylineCountChanged
            )
            .step(100.0),
        ]
        .spacing(10)
        .padding(5);

        // Points per polyline slider
        let points_slider = row![
            text(format!("Points/Polyline: {}", self.points_per_polyline))
                .size(14)
                .width(150),
            Slider::new(
                2.0..=1000.0,
                self.points_per_polyline as f32,
                Message::PointsPerPolylineChanged
            )
            .step(10.0),
        ]
        .spacing(10)
        .padding(5);

        // Regenerate button
        let regenerate_btn = button("Regenerate Shapes")
            .on_press(Message::Regenerate)
            .padding(10);

        // Controls panel
        let controls = column![
            text("Stress Test Controls").size(18),
            rectangle_slider,
            polyline_slider,
            points_slider,
            regenerate_btn,
        ]
        .spacing(5)
        .padding(10);

        // Main layout
        column![fps_display, stats_display, controls, chart,]
            .spacing(10)
            .padding(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::window::frames().map(Message::Tick)
    }

    const fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .subscription(Self::subscription)
            .antialiasing(true)
            .run()
    }
}

fn main() -> iced::Result {
    StressTestApp::run()
}
