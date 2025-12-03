//! Chart Stress Test
//!
//! A stress-testing example for the chart widget with:
//! - FPS counter to monitor performance
//! - GUI-configurable stress parameters
//! - Toggles for Fill vs Stroke (testing Hybrid Engine optimizations)
//! - Pre-calculated geometry to isolate rendering performance
//! - Advanced styling controls (Size, Opacity, Stroke Style)
//! - **View-Aware Generation**: Shapes generate within the current pan/zoom bounds.

use std::time::Instant;

use aksel::{PlotPoint, Scale, scale::Linear}; // Added Scale trait import
use iced::{
    Alignment, Color, Element, Point, Subscription, Task, Theme,
    mouse::ScrollDelta,
    widget::{Slider, button, checkbox, column, radio, row, text},
};
use iced_aksel::{
    Axis, Chart, DragDelta, Length, Plot, State, Stroke, StrokeStyle,
    axis::{self, Position},
    plot,
    shape::{self, Circle, Rectangle},
};
use rand::Rng;

const AXIS_ID_X: &str = "x";
const AXIS_ID_Y: &str = "y";

type AxisId = &'static str;

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    // Counts
    RectangleCountChanged(f32),
    CircleCountChanged(f32),
    // Geometry Generation
    MinSizeChanged(f32),
    MaxSizeChanged(f32),
    OpacityChanged(f32),
    // Rendering Styles
    ToggleFill(bool),
    ToggleStroke(bool),
    StrokeWidthChanged(f32),
    StrokeStyleChanged(StrokeStyle),
    // Actions
    Regenerate,
    // Chart interaction
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),
}

// --- Layers ---

struct StressRectangles {
    // We store the specific shape struct directly to avoid geometry math in the draw loop
    geometry: Vec<Rectangle<f64>>,
    // We store colors separately so we can apply them during draw
    colors: Vec<Color>,

    // Render-time render properties
    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressRectangles {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        // Zip geometry with colors to avoid lookups
        for (base_rect, &color) in self.geometry.iter().zip(self.colors.iter()) {
            // Clone the geometry (cheap: just f64s)
            let mut rect = base_rect.clone();

            // Apply styles Just-In-Time based on toggles
            if self.show_fill {
                rect = rect.fill(color);
            }

            if self.show_stroke {
                // Use a standard white stroke for contrast, with configurable width/style
                rect = rect.stroke(
                    Stroke::new(Color::WHITE, Length::Screen(self.stroke_width))
                        .with_style(self.stroke_style),
                );
            }

            // Only add to the render list if it contributes pixels
            if self.show_fill || self.show_stroke {
                plot.add_shape(rect);
            }
        }
    }
}

struct StressCircles {
    geometry: Vec<Circle<f64>>,
    colors: Vec<Color>,

    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressCircles {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        for (base_circle, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut circle = base_circle.clone();

            if self.show_fill {
                circle = circle.fill(color);
            }

            if self.show_stroke {
                circle = circle.stroke(
                    Stroke::new(Color::WHITE, Length::Screen(self.stroke_width))
                        .with_style(self.stroke_style),
                );
            }

            if self.show_fill || self.show_stroke {
                plot.add_shape(circle);
            }
        }
    }
}

// --- App ---

struct StressTestApp {
    state: State<AxisId, f64>,
    rectangles_layer: StressRectangles,
    circles_layer: StressCircles,

    // Generation Configuration
    rectangle_count: usize,
    circle_count: usize,
    min_size: f32,
    max_size: f32,
    opacity: f32,

    // FPS counter
    last_frame_time: Option<Instant>,
    fps: f32,
    frame_times: Vec<f32>,
}

impl StressTestApp {
    fn init() -> (Self, Task<Message>) {
        let mut state: State<AxisId, f64> = State::new();

        state.set_axis(
            AXIS_ID_X,
            Axis::new(Linear::new(0.0, 1000.0), Position::Bottom),
        );
        state.set_axis(
            AXIS_ID_Y,
            Axis::new(Linear::new(0.0, 1000.0), Position::Left),
        );

        let mut app = Self {
            state,
            rectangles_layer: StressRectangles {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_style: StrokeStyle::Solid,
            },
            circles_layer: StressCircles {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_style: StrokeStyle::Solid,
            },
            rectangle_count: 100,
            circle_count: 100,
            min_size: 5.0,
            max_size: 20.0,
            opacity: 1.0,
            last_frame_time: None,
            fps: 0.0,
            frame_times: Vec::with_capacity(60),
        };

        app.generate_shapes();

        (app, Task::none())
    }

    fn generate_shapes(&mut self) {
        let mut rng = rand::rng();

        // 1. Get current View Bounds from State
        // This ensures we generate shapes where the camera currently IS.
        let (x_min, x_max) = self
            .state
            .get_axis(&AXIS_ID_X)
            .map(|axis| {
                let (min, max) = axis.scale().domain();
                // Ensure correct ordering for RNG
                if min <= max {
                    (*min, *max)
                } else {
                    (*max, *min)
                }
            })
            .unwrap_or((0.0, 1000.0));

        let (y_min, y_max) = self
            .state
            .get_axis(&AXIS_ID_X)
            .map(|axis| {
                let (min, max) = axis.scale().domain();
                if min <= max {
                    (*min, *max)
                } else {
                    (*max, *min)
                }
            })
            .unwrap_or((0.0, 1000.0));

        // 2. Rectangles
        self.rectangles_layer.geometry.clear();
        self.rectangles_layer.colors.clear();
        self.rectangles_layer.geometry.reserve(self.rectangle_count);
        self.rectangles_layer.colors.reserve(self.rectangle_count);

        for _ in 0..self.rectangle_count {
            // Generate within current view
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);

            let width =
                rng.random_range(self.min_size..self.max_size.max(self.min_size + 1.0)) as f64;
            let height =
                rng.random_range(self.min_size..self.max_size.max(self.min_size + 1.0)) as f64;

            // Pre-calculate geometry
            let rect = Rectangle::from_corners(
                PlotPoint::new(x, y),
                PlotPoint::new(x + width, y + height),
            );

            self.rectangles_layer.geometry.push(rect);
            self.rectangles_layer.colors.push(Color::from_rgba(
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                self.opacity,
            ));
        }

        // 3. Circles
        self.circles_layer.geometry.clear();
        self.circles_layer.colors.clear();
        self.circles_layer.geometry.reserve(self.circle_count);
        self.circles_layer.colors.reserve(self.circle_count);

        for _ in 0..self.circle_count {
            // Generate within current view
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);

            // Radius is roughly half size
            let radius = rng.random_range(
                (self.min_size / 2.0)..(self.max_size / 2.0).max(self.min_size / 2.0 + 0.1),
            );

            // Pre-calculate geometry
            let circle = Circle::new(PlotPoint::new(x, y), Length::Plot(radius as f64));

            self.circles_layer.geometry.push(circle);
            self.circles_layer.colors.push(Color::from_rgba(
                rng.random_range(0.5..1.0),
                rng.random_range(0.2..0.8),
                rng.random_range(0.2..0.8),
                self.opacity,
            ));
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                if let Some(last) = self.last_frame_time {
                    let delta = now.duration_since(last).as_secs_f32();
                    if delta > 0.0 {
                        let instant_fps = 1.0 / delta;
                        self.fps = self.fps * 0.9 + instant_fps * 0.1;
                        self.frame_times.push(delta * 1000.0);
                        if self.frame_times.len() > 60 {
                            self.frame_times.remove(0);
                        }
                    }
                }
                self.last_frame_time = Some(now);
                Task::none()
            }
            // Generation Parameters (Trigger Regenerate)
            Message::RectangleCountChanged(v) => {
                self.rectangle_count = v as usize;
                self.generate_shapes();
                Task::none()
            }
            Message::CircleCountChanged(v) => {
                self.circle_count = v as usize;
                self.generate_shapes();
                Task::none()
            }
            Message::MinSizeChanged(v) => {
                self.min_size = v;
                if self.min_size > self.max_size {
                    self.max_size = self.min_size;
                }
                self.generate_shapes();
                Task::none()
            }
            Message::MaxSizeChanged(v) => {
                self.max_size = v;
                if self.max_size < self.min_size {
                    self.min_size = self.max_size;
                }
                self.generate_shapes();
                Task::none()
            }
            Message::OpacityChanged(v) => {
                self.opacity = v;
                self.generate_shapes();
                Task::none()
            }
            // Render Parameters (Instant Update)
            Message::ToggleFill(v) => {
                self.rectangles_layer.show_fill = v;
                self.circles_layer.show_fill = v;
                Task::none()
            }
            Message::ToggleStroke(v) => {
                self.rectangles_layer.show_stroke = v;
                self.circles_layer.show_stroke = v;
                Task::none()
            }
            Message::StrokeWidthChanged(v) => {
                self.rectangles_layer.stroke_width = v;
                self.circles_layer.stroke_width = v;
                Task::none()
            }
            Message::StrokeStyleChanged(v) => {
                self.rectangles_layer.stroke_style = v;
                self.circles_layer.stroke_style = v;
                Task::none()
            }
            Message::Regenerate => {
                self.generate_shapes();
                Task::none()
            }

            Message::ChartDragged(delta) => {
                let x = delta.x as f64;
                let y = delta.y as f64;
                self.state.pan_scales(AXIS_ID_X, AXIS_ID_Y, x, y);
                Task::none()
            }

            Message::ChartScrolled(point, delta) => {
                if let ScrollDelta::Lines { x: _, y } = delta {
                    let factor = 1.1f64.powf(y.into());
                    self.state.zoom_scales(
                        AXIS_ID_X,
                        AXIS_ID_Y,
                        point.x.into(),
                        point.y.into(),
                        factor,
                    );
                };
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .layer(&self.rectangles_layer, AXIS_ID_X, AXIS_ID_Y)
            .layer(&self.circles_layer, AXIS_ID_X, AXIS_ID_Y)
            .on_drag(Message::ChartDragged)
            .on_scroll(Message::ChartScrolled);

        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        } else {
            0.0
        };

        // --- Controls Sections ---

        // 1. Stats
        let stats_row = row![
            text(format!("FPS: {:.0}", self.fps)).size(24),
            text(format!("Frame Time: {:.2}ms", avg_frame_time)).size(16),
            text(format!(
                "Vertices: ~{}",
                (self.rectangle_count
                    * (if self.rectangles_layer.show_fill {
                        4
                    } else {
                        0
                    } + if self.rectangles_layer.show_stroke {
                        8
                    } else {
                        0
                    }))
                    + (self.circle_count * 100)
            ))
            .size(16),
        ]
        .spacing(20);

        // 2. Count Sliders
        let counts_col = column![
            row![
                text(format!("Rects: {}", self.rectangle_count)).width(100),
                Slider::new(
                    0.0..=50000.0,
                    self.rectangle_count as f32,
                    Message::RectangleCountChanged
                )
                .step(500.0)
            ]
            .spacing(10),
            row![
                text(format!("Circles: {}", self.circle_count)).width(100),
                Slider::new(
                    0.0..=10000.0,
                    self.circle_count as f32,
                    Message::CircleCountChanged
                )
                .step(100.0)
            ]
            .spacing(10),
        ]
        .spacing(5)
        .padding(5);

        // 3. Geometry Sliders (Size/Opacity)
        let geometry_col = column![
            row![
                text(format!("Min Size: {:.0}", self.min_size)).width(100),
                Slider::new(1.0..=200.0, self.min_size, Message::MinSizeChanged)
            ]
            .spacing(10),
            row![
                text(format!("Max Size: {:.0}", self.max_size)).width(100),
                Slider::new(1.0..=200.0, self.max_size, Message::MaxSizeChanged)
            ]
            .spacing(10),
            row![
                text(format!("Opacity: {:.1}", self.opacity)).width(100),
                Slider::new(0.0..=1.0, self.opacity, Message::OpacityChanged).step(0.05)
            ]
            .spacing(10),
        ]
        .spacing(5)
        .padding(5);

        // 4. Style Controls
        let stroke_style_row = row![
            radio(
                "Solid",
                StrokeStyle::Solid,
                Some(self.rectangles_layer.stroke_style),
                Message::StrokeStyleChanged
            ),
            radio(
                "Dashed",
                StrokeStyle::Dashed,
                Some(self.rectangles_layer.stroke_style),
                Message::StrokeStyleChanged
            ),
            radio(
                "Dotted",
                StrokeStyle::Dotted,
                Some(self.rectangles_layer.stroke_style),
                Message::StrokeStyleChanged
            ),
        ]
        .spacing(15);

        let styles_col = column![
            // Fixed: Checkboxes now use a row with text for labels
            row![
                row![
                    checkbox(self.rectangles_layer.show_fill).on_toggle(Message::ToggleFill),
                    text("Fill"),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
                row![
                    checkbox(self.rectangles_layer.show_stroke).on_toggle(Message::ToggleStroke),
                    text("Stroke"),
                ]
                .spacing(5)
                .align_y(Alignment::Center),
            ]
            .spacing(20),
            row![
                text(format!(
                    "Stroke Width: {:.1}",
                    self.rectangles_layer.stroke_width
                ))
                .width(120),
                Slider::new(
                    0.5..=10.0,
                    self.rectangles_layer.stroke_width,
                    Message::StrokeWidthChanged
                )
                .step(0.5)
            ]
            .spacing(10),
            stroke_style_row,
        ]
        .spacing(5)
        .padding(5);

        // Combine controls in a responsive grid/row
        let controls_row = row![
            counts_col,
            geometry_col,
            styles_col,
            button("Regenerate")
                .on_press(Message::Regenerate)
                .padding(10)
        ]
        .spacing(20);

        column![stats_row, controls_row, chart]
            .spacing(10)
            .padding(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::window::frames().map(Message::Tick)
    }

    fn theme(&self) -> Theme {
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

pub fn main() -> iced::Result {
    StressTestApp::run()
}
