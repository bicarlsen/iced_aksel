//! Chart Stress Test
//!
//! A stress-testing example for the chart widget with:
//! - FPS counter to monitor performance
//! - GUI-configurable stress parameters
//! - Toggles for Fill vs Stroke (testing Hybrid Engine optimizations)
//! - Pre-calculated geometry to isolate rendering performance
//! - Advanced styling controls (Size, Opacity, Stroke Style)
//! - **View-Aware Generation**: Shapes generate within the current pan/zoom bounds.
//! - **Length Modes**: Switch between Screen (px) and Plot (data) units for sizes and strokes.

use std::time::Instant;

use aksel::{PlotPoint, Scale, scale::Linear};
use iced::{
    Alignment, Color, Element, Point, Subscription, Task, Theme,
    mouse::ScrollDelta,
    widget::{Slider, button, checkbox, column, radio, row, text},
};
use iced_aksel::{
    Axis, Chart, DragDelta, Length, Plot, State, Stroke, StrokeStyle,
    axis::{self, Position},
    plot,
    shape::{self, Circle, Rectangle, Triangle},
};
use rand::Rng;

const AXIS_ID_X: &str = "x";
const AXIS_ID_Y: &str = "y";

type AxisId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthMode {
    Screen,
    Plot,
}

impl std::fmt::Display for LengthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LengthMode::Screen => write!(f, "Screen (px)"),
            LengthMode::Plot => write!(f, "Plot (units)"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    // Counts
    RectangleCountChanged(f32),
    CircleCountChanged(f32),
    TriangleCountChanged(f32),
    // Geometry Generation
    MinSizeChanged(f32),
    MaxSizeChanged(f32),
    OpacityChanged(f32),
    SizeModeChanged(LengthMode),
    // Rendering Styles
    ToggleFill(bool),
    ToggleStroke(bool),
    StrokeWidthChanged(f32),
    StrokeWidthModeChanged(LengthMode),
    StrokeStyleChanged(StrokeStyle),
    // Actions
    Regenerate,
    // Chart interaction
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),
}

// --- Layers ---

struct StressRectangles {
    geometry: Vec<Rectangle<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressRectangles {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        for (base_rect, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut rect = base_rect.clone();

            if self.show_fill {
                rect = rect.fill(color);
            }

            if self.show_stroke {
                let thickness = match self.stroke_width_mode {
                    LengthMode::Screen => Length::Screen(self.stroke_width),
                    LengthMode::Plot => Length::Plot(self.stroke_width as f64),
                };
                rect =
                    rect.stroke(Stroke::new(Color::WHITE, thickness).with_style(self.stroke_style));
            }

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
    stroke_width_mode: LengthMode,
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
                let thickness = match self.stroke_width_mode {
                    LengthMode::Screen => Length::Screen(self.stroke_width),
                    LengthMode::Plot => Length::Plot(self.stroke_width as f64),
                };
                circle = circle
                    .stroke(Stroke::new(Color::WHITE, thickness).with_style(self.stroke_style));
            }

            if self.show_fill || self.show_stroke {
                plot.add_shape(circle);
            }
        }
    }
}

struct StressTriangles {
    geometry: Vec<Triangle<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressTriangles {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        for (base_tri, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut tri = base_tri.clone();

            if self.show_fill {
                tri = tri.fill(color);
            }

            if self.show_stroke {
                let thickness = match self.stroke_width_mode {
                    LengthMode::Screen => Length::Screen(self.stroke_width),
                    LengthMode::Plot => Length::Plot(self.stroke_width as f64),
                };
                tri =
                    tri.stroke(Stroke::new(Color::WHITE, thickness).with_style(self.stroke_style));
            }

            if self.show_fill || self.show_stroke {
                plot.add_shape(tri);
            }
        }
    }
}

// --- App ---

struct StressTestApp {
    state: State<AxisId, f64>,
    rectangles_layer: StressRectangles,
    circles_layer: StressCircles,
    triangles_layer: StressTriangles,

    // Generation Configuration
    rectangle_count: usize,
    circle_count: usize,
    triangle_count: usize,
    min_size: f32,
    max_size: f32,
    opacity: f32,
    size_mode: LengthMode,

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
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            circles_layer: StressCircles {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            triangles_layer: StressTriangles {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            rectangle_count: 5000,
            circle_count: 5000,
            triangle_count: 5000,
            min_size: 5.0,
            max_size: 20.0,
            opacity: 1.0,
            size_mode: LengthMode::Screen,
            last_frame_time: None,
            fps: 0.0,
            frame_times: Vec::with_capacity(60),
        };

        app.generate_shapes();

        (app, Task::none())
    }

    fn generate_shapes(&mut self) {
        let mut rng = rand::rng();

        // 1. Get current View Bounds
        let (x_min, x_max) = self
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

        let (y_min, y_max) = self
            .state
            .get_axis(&AXIS_ID_Y)
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
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            let w_val =
                rng.random_range(self.min_size..self.max_size.max(self.min_size + 1.0)) as f64;
            let h_val =
                rng.random_range(self.min_size..self.max_size.max(self.min_size + 1.0)) as f64;

            let center = PlotPoint::new(x, y);
            let (width, height) = match self.size_mode {
                LengthMode::Screen => (Length::Screen(w_val as f32), Length::Screen(h_val as f32)),
                LengthMode::Plot => (Length::Plot(w_val), Length::Plot(h_val)),
            };

            self.rectangles_layer
                .geometry
                .push(Rectangle::new(center, width, height));
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
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            let r_val = rng.random_range(
                (self.min_size / 2.0)..(self.max_size / 2.0).max(self.min_size / 2.0 + 0.1),
            ) as f64;

            let radius = match self.size_mode {
                LengthMode::Screen => Length::Screen(r_val as f32),
                LengthMode::Plot => Length::Plot(r_val),
            };

            self.circles_layer
                .geometry
                .push(Circle::new(PlotPoint::new(x, y), radius));
            self.circles_layer.colors.push(Color::from_rgba(
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                self.opacity,
            ));
        }

        // 4. Triangles
        self.triangles_layer.geometry.clear();
        self.triangles_layer.colors.clear();
        self.triangles_layer.geometry.reserve(self.triangle_count);
        self.triangles_layer.colors.reserve(self.triangle_count);

        for _ in 0..self.triangle_count {
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            let r_val = rng.random_range(
                (self.min_size / 2.0)..(self.max_size / 2.0).max(self.min_size / 2.0 + 0.1),
            ) as f64;

            let radius = match self.size_mode {
                LengthMode::Screen => Length::Screen(r_val as f32),
                LengthMode::Plot => Length::Plot(r_val),
            };

            // Using Equilateral triangle constructor for markers
            self.triangles_layer
                .geometry
                .push(Triangle::equilateral(PlotPoint::new(x, y), radius));
            self.triangles_layer.colors.push(Color::from_rgba(
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
                rng.random_range(0.0..1.0),
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
            // Generation Parameters
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
            Message::TriangleCountChanged(v) => {
                self.triangle_count = v as usize;
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
            Message::SizeModeChanged(mode) => {
                self.size_mode = mode;
                self.generate_shapes();
                Task::none()
            }
            // Render Parameters
            Message::ToggleFill(v) => {
                self.rectangles_layer.show_fill = v;
                self.circles_layer.show_fill = v;
                self.triangles_layer.show_fill = v;
                Task::none()
            }
            Message::ToggleStroke(v) => {
                self.rectangles_layer.show_stroke = v;
                self.circles_layer.show_stroke = v;
                self.triangles_layer.show_stroke = v;
                Task::none()
            }
            Message::StrokeWidthChanged(v) => {
                self.rectangles_layer.stroke_width = v;
                self.circles_layer.stroke_width = v;
                self.triangles_layer.stroke_width = v;
                Task::none()
            }
            Message::StrokeWidthModeChanged(mode) => {
                self.rectangles_layer.stroke_width_mode = mode;
                self.circles_layer.stroke_width_mode = mode;
                self.triangles_layer.stroke_width_mode = mode;
                Task::none()
            }
            Message::StrokeStyleChanged(v) => {
                self.rectangles_layer.stroke_style = v;
                self.circles_layer.stroke_style = v;
                self.triangles_layer.stroke_style = v;
                Task::none()
            }
            Message::Regenerate => {
                self.generate_shapes();
                Task::none()
            }
            // Chart
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
            .layer(&self.triangles_layer, AXIS_ID_X, AXIS_ID_Y)
            .on_drag(Message::ChartDragged)
            .on_scroll(Message::ChartScrolled);

        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        } else {
            0.0
        };

        // --- Controls ---

        // 1. Stats
        let stats_row = row![
            text(format!("FPS: {:.0}", self.fps)).size(24),
            text(format!("Frame Time: {:.2}ms", avg_frame_time)).size(16),
            text(format!(
                "Total Objects: {}",
                self.rectangle_count + self.circle_count + self.triangle_count
            ))
            .size(16),
        ]
        .spacing(20);

        // 2. Count Sliders (Range Increased to 500,000)
        let counts_col = column![
            row![
                text(format!("Rects: {}", self.rectangle_count)).width(100),
                Slider::new(
                    0.0..=500000.0,
                    self.rectangle_count as f32,
                    Message::RectangleCountChanged
                )
                .step(5000.0)
            ]
            .spacing(10),
            row![
                text(format!("Circles: {}", self.circle_count)).width(100),
                Slider::new(
                    0.0..=500000.0,
                    self.circle_count as f32,
                    Message::CircleCountChanged
                )
                .step(5000.0)
            ]
            .spacing(10),
            row![
                text(format!("Triangles: {}", self.triangle_count)).width(100),
                Slider::new(
                    0.0..=500000.0,
                    self.triangle_count as f32,
                    Message::TriangleCountChanged
                )
                .step(5000.0)
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
            row![
                text("Size Mode:").size(14),
                radio(
                    "Screen",
                    LengthMode::Screen,
                    Some(self.size_mode),
                    Message::SizeModeChanged
                ),
                radio(
                    "Plot",
                    LengthMode::Plot,
                    Some(self.size_mode),
                    Message::SizeModeChanged
                ),
            ]
            .spacing(15)
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

        let stroke_mode_row = row![
            text("Width Mode:").size(14),
            radio(
                "Screen",
                LengthMode::Screen,
                Some(self.rectangles_layer.stroke_width_mode),
                Message::StrokeWidthModeChanged
            ),
            radio(
                "Plot",
                LengthMode::Plot,
                Some(self.rectangles_layer.stroke_width_mode),
                Message::StrokeWidthModeChanged
            ),
        ]
        .spacing(15);

        let styles_col = column![
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
                    0.5..=20.0,
                    self.rectangles_layer.stroke_width,
                    Message::StrokeWidthChanged
                )
                .step(0.5)
            ]
            .spacing(10),
            stroke_mode_row,
            stroke_style_row,
        ]
        .spacing(5)
        .padding(5);

        // Combine controls
        let controls_row = row![counts_col, geometry_col, styles_col,]
            .spacing(20)
            .align_y(Alignment::Start);

        let regenerate_btn = button(
            text("Regenerate Shapes")
                .width(iced::Length::Fill)
                .align_x(Alignment::Center),
        )
        .on_press(Message::Regenerate)
        .padding(10)
        .width(iced::Length::Fill);

        column![stats_row, controls_row, regenerate_btn, chart]
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
