//! Chart shape layer testing suite
use std::fmt::Display;

use aksel::{PlotPoint, scale::Linear};
use iced::{
    Color, Element, Length as UiLength, Point, Task, Theme,
    mouse::ScrollDelta,
    widget::{
        Space, button, checkbox, column, combo_box, container, pick_list, row, slider, text,
        text_input,
    },
};
use iced_aksel::{Axis, Chart, Length, Plot, State, Stroke, axis, plot, shape};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

const X_ID: &str = "linear_x";
const Y_ID: &str = "linear_y";

type AxisId = &'static str;

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application State ---

struct ExampleApp {
    chart_state: State<AxisId, f64>,

    // UI State
    shapes_combo: combo_box::State<ShapeType>,

    // Manual Input State for Colors (Strings)
    input_stroke_r: String,
    input_stroke_g: String,
    input_stroke_b: String,

    input_fill_r: String,
    input_fill_g: String,
    input_fill_b: String,

    // Data & Configuration
    env: TestingEnvironment,
}

#[derive(Debug, Clone)]
enum Message {
    // Chart Interactions
    ChartClicked(Point<f32>),
    ChartScrolled(Point<f32>, ScrollDelta),
    ClearShapes,
    UndoLastShape,

    // Configuration Changes
    ShapeChanged(ShapeType),

    // Size Changes
    SizeChanged(f32),
    SizeUnitChanged(UnitType),

    // Sector Specifics
    SectorStartAngleChanged(f32),
    SectorEndAngleChanged(f32),
    SectorInnerRadiusChanged(f32),

    // Style Changes
    StrokeWidthChanged(f32),
    StrokeWidthUnitChanged(UnitType),
    FillEnabledChanged(bool),

    // Color Input Changes
    StrokeColorRChanged(String),
    StrokeColorGChanged(String),
    StrokeColorBChanged(String),
    FillColorRChanged(String),
    FillColorGChanged(String),
    FillColorBChanged(String),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut chart_state = State::new();

        // Initialize axes 0-100
        chart_state.set_axis(
            X_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom),
        );
        chart_state.set_axis(
            Y_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Right),
        );

        let env = TestingEnvironment::new();

        (
            Self {
                chart_state,
                shapes_combo: combo_box::State::new(ShapeType::iter().collect()),
                input_stroke_r: "0.0".to_string(),
                input_stroke_g: "0.0".to_string(),
                input_stroke_b: "0.0".to_string(),
                input_fill_r: "1.0".to_string(),
                input_fill_g: "1.0".to_string(),
                input_fill_b: "1.0".to_string(),
                env,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChartClicked(point) => {
                if let (Some(x_axis), Some(y_axis)) = (
                    self.chart_state.get_axis(&X_ID),
                    self.chart_state.get_axis(&Y_ID),
                ) {
                    let plot_point = PlotPoint::new(
                        x_axis.denormalize(point.x as f64),
                        y_axis.denormalize(point.y as f64),
                    );

                    self.env.add_shape_at(plot_point);
                }
                Task::none()
            }
            Message::ChartScrolled(point, delta) => {
                let y = match delta {
                    ScrollDelta::Lines { y, .. } => y,
                    ScrollDelta::Pixels { y, .. } => y / 15.0,
                };

                let factor = 1.05_f64.powf(y as f64);

                if let Some(axis) = self.chart_state.get_axis_mut(&X_ID) {
                    axis.scale_mut().zoom(factor, Some(point.x as f64));
                }
                if let Some(axis) = self.chart_state.get_axis_mut(&Y_ID) {
                    axis.scale_mut().zoom(factor, Some(point.y as f64));
                }
                Task::none()
            }
            Message::ClearShapes => {
                self.env.drawn_shapes.clear();
                Task::none()
            }
            Message::UndoLastShape => {
                self.env.drawn_shapes.pop();
                Task::none()
            }
            Message::ShapeChanged(shape) => {
                self.env.config.selected_shape = shape;
                Task::none()
            }
            // Size Updates
            Message::SizeChanged(size) => {
                self.env.config.shape_size = size;
                Task::none()
            }
            Message::SizeUnitChanged(unit) => {
                self.env.config.shape_size_unit = unit;
                Task::none()
            }
            // Sector Updates
            Message::SectorStartAngleChanged(angle) => {
                self.env.config.sector_start = angle;
                Task::none()
            }
            Message::SectorEndAngleChanged(angle) => {
                self.env.config.sector_end = angle;
                Task::none()
            }
            Message::SectorInnerRadiusChanged(r) => {
                self.env.config.sector_inner = r;
                Task::none()
            }
            // Style Updates
            Message::StrokeWidthChanged(width) => {
                self.env.config.stroke_width = width;
                Task::none()
            }
            Message::StrokeWidthUnitChanged(unit) => {
                self.env.config.stroke_width_unit = unit;
                Task::none()
            }
            Message::FillEnabledChanged(enabled) => {
                self.env.config.fill_enabled = enabled;
                Task::none()
            }
            // Color Updates
            Message::StrokeColorRChanged(val) => {
                self.input_stroke_r = val;
                if let Ok(r) = self.input_stroke_r.parse::<f32>() {
                    self.env.config.stroke_color.r = (r / 255.0).clamp(0.0, 1.0);
                }
                Task::none()
            }
            Message::StrokeColorGChanged(val) => {
                self.input_stroke_g = val;
                if let Ok(g) = self.input_stroke_g.parse::<f32>() {
                    self.env.config.stroke_color.g = (g / 255.0).clamp(0.0, 1.0);
                }
                Task::none()
            }
            Message::StrokeColorBChanged(val) => {
                self.input_stroke_b = val;
                if let Ok(b) = self.input_stroke_b.parse::<f32>() {
                    self.env.config.stroke_color.b = (b / 255.0).clamp(0.0, 1.0);
                }
                Task::none()
            }
            Message::FillColorRChanged(val) => {
                self.input_fill_r = val;
                if let Ok(r) = self.input_fill_r.parse::<f32>() {
                    self.env.config.fill_color.r = (r / 255.0).clamp(0.0, 1.0);
                }
                Task::none()
            }
            Message::FillColorGChanged(val) => {
                self.input_fill_g = val;
                if let Ok(g) = self.input_fill_g.parse::<f32>() {
                    self.env.config.fill_color.g = (g / 255.0).clamp(0.0, 1.0);
                }
                Task::none()
            }
            Message::FillColorBChanged(val) => {
                self.input_fill_b = val;
                if let Ok(b) = self.input_fill_b.parse::<f32>() {
                    self.env.config.fill_color.b = (b / 255.0).clamp(0.0, 1.0);
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- Control Panel ---
        let mut controls = column![
            text("Drawing Tool").size(18),
            Space::new().height(UiLength::Fixed(10.0)),
            // Tool Selection
            container(column![
                text("Tool:").size(14),
                combo_box(
                    &self.shapes_combo,
                    "Select Tool",
                    Some(&self.env.config.selected_shape),
                    Message::ShapeChanged
                )
                .width(UiLength::Fill),
            ])
            .style(|_| container::Style {
                background: Some(Color::from_rgb(0.15, 0.15, 0.15).into()),
                ..Default::default()
            })
            .padding(10),
            Space::new().height(UiLength::Fixed(10.0)),
        ];

        // --- Shape Specific Settings ---
        let mut shape_settings = column![text("Shape Settings").size(14)];

        // Common Size
        shape_settings = shape_settings.push(column![
            text(format!(
                "Size (Radius/Width): {:.1}",
                self.env.config.shape_size
            )),
            row![
                slider(
                    1.0..=100.0,
                    self.env.config.shape_size,
                    Message::SizeChanged
                )
                .step(0.5),
                pick_list(
                    UnitType::iter().collect::<Vec<_>>(),
                    Some(self.env.config.shape_size_unit),
                    Message::SizeUnitChanged
                )
                .width(UiLength::Shrink)
            ]
            .spacing(5)
        ]);

        // Specifics for Sector
        if self.env.config.selected_shape == ShapeType::Sector {
            shape_settings = shape_settings.push(Space::new().height(10));
            shape_settings = shape_settings.push(text("Sector Angles (Deg)"));
            shape_settings = shape_settings.push(
                row![
                    column![
                        text(format!("Start: {:.0}", self.env.config.sector_start)).size(12),
                        slider(
                            0.0..=360.0,
                            self.env.config.sector_start,
                            Message::SectorStartAngleChanged
                        ),
                    ],
                    column![
                        text(format!("End: {:.0}", self.env.config.sector_end)).size(12),
                        slider(
                            0.0..=360.0,
                            self.env.config.sector_end,
                            Message::SectorEndAngleChanged
                        ),
                    ]
                ]
                .spacing(10),
            );
            shape_settings = shape_settings.push(column![
                text(format!(
                    "Inner Radius %: {:.0}",
                    self.env.config.sector_inner
                ))
                .size(12),
                slider(
                    0.0..=90.0,
                    self.env.config.sector_inner,
                    Message::SectorInnerRadiusChanged
                ),
            ]);
        }

        controls = controls.push(
            container(shape_settings)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.15, 0.15, 0.15).into()),
                    ..Default::default()
                })
                .padding(10),
        );

        controls = controls.push(Space::new().height(UiLength::Fixed(10.0)));

        // --- Style Settings ---
        let style_settings = column![
            text("Style Settings").size(14),
            // Stroke
            text(format!("Stroke Width: {:.1}", self.env.config.stroke_width)),
            row![
                slider(
                    0.0..=20.0,
                    self.env.config.stroke_width,
                    Message::StrokeWidthChanged
                )
                .step(0.1),
                pick_list(
                    UnitType::iter().collect::<Vec<_>>(),
                    Some(self.env.config.stroke_width_unit),
                    Message::StrokeWidthUnitChanged
                )
                .width(UiLength::Shrink)
            ]
            .spacing(5),
            text("Stroke Color (RGB):"),
            row![
                text_input("R", &self.input_stroke_r)
                    .on_input(Message::StrokeColorRChanged)
                    .width(UiLength::Fill),
                text_input("G", &self.input_stroke_g)
                    .on_input(Message::StrokeColorGChanged)
                    .width(UiLength::Fill),
                text_input("B", &self.input_stroke_b)
                    .on_input(Message::StrokeColorBChanged)
                    .width(UiLength::Fill),
            ]
            .spacing(5),
            Space::new().height(10),
            // Fill
            checkbox(self.env.config.fill_enabled).on_toggle(Message::FillEnabledChanged),
            text("Fill Color (RGB):"),
            row![
                text_input("R", &self.input_fill_r)
                    .on_input(Message::FillColorRChanged)
                    .width(UiLength::Fill),
                text_input("G", &self.input_fill_g)
                    .on_input(Message::FillColorGChanged)
                    .width(UiLength::Fill),
                text_input("B", &self.input_fill_b)
                    .on_input(Message::FillColorBChanged)
                    .width(UiLength::Fill),
            ]
            .spacing(5),
        ];

        controls = controls.push(
            container(style_settings)
                .style(|_| container::Style {
                    background: Some(Color::from_rgb(0.15, 0.15, 0.15).into()),
                    ..Default::default()
                })
                .padding(10),
        );

        controls = controls.push(Space::new().height(UiLength::Fixed(20.0)));

        // --- Actions ---
        controls = controls.push(
            row![
                button("Undo")
                    .on_press(Message::UndoLastShape)
                    .width(UiLength::Fill),
                button("Clear All")
                    .on_press(Message::ClearShapes)
                    .width(UiLength::Fill),
            ]
            .spacing(10),
        );

        controls = controls.padding(10).spacing(10).width(300);

        // --- Chart ---
        let chart = Chart::new(&self.chart_state)
            .layer(&self.env, X_ID, Y_ID)
            .on_click(Message::ChartClicked)
            .on_scroll(Message::ChartScrolled);

        // --- Layout ---
        container(row![controls, chart]).padding(20).into()
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Theme::Dark)
            .antialiasing(true)
            .run()
    }
}

// --- Enums ---

#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
enum ShapeType {
    Rectangle,
    Circle,
    Triangle,
    Sector,
    Polygon, // Used as Diamond Marker for now
}

impl Display for ShapeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShapeType::Polygon => write!(f, "Strange Polygon (Irregular)"),
            ShapeType::Sector => write!(f, "Sector (Pie/Donut)"),
            _ => write!(f, "{:?}", self),
        }
    }
}

#[derive(EnumIter, Debug, Clone, Copy, PartialEq, Eq)]
enum UnitType {
    Screen,
    Plot,
}

impl Display for UnitType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitType::Screen => write!(f, "Screen"),
            UnitType::Plot => write!(f, "Plot"),
        }
    }
}

// --- Tool Configuration (The Brush) ---

#[derive(Debug, Clone)]
struct ToolConfig {
    selected_shape: ShapeType,
    shape_size: f32,
    shape_size_unit: UnitType,

    // Sector Specifics
    sector_start: f32,
    sector_end: f32,
    sector_inner: f32, // Percent (0-90)

    stroke_width: f32,
    stroke_width_unit: UnitType,
    stroke_color: Color,

    fill_enabled: bool,
    fill_color: Color,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            selected_shape: ShapeType::Rectangle,
            shape_size: 20.0,
            shape_size_unit: UnitType::Plot,

            sector_start: 0.0,
            sector_end: 270.0,
            sector_inner: 0.0,

            stroke_width: 2.0,
            stroke_width_unit: UnitType::Screen,
            stroke_color: Color::BLACK,

            fill_enabled: true,
            fill_color: Color::WHITE,
        }
    }
}

// --- Data Handler / Environment ---

struct PlacedShape {
    shape_type: ShapeType,
    position: PlotPoint,
}

struct TestingEnvironment {
    // Current Settings (The "Brush")
    pub config: ToolConfig,

    // Persistent Storage
    pub drawn_shapes: Vec<PlacedShape>,
}

impl TestingEnvironment {
    fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            drawn_shapes: Vec::new(),
        }
    }

    fn add_shape_at(&mut self, point: PlotPoint) {
        self.drawn_shapes.push(PlacedShape {
            shape_type: self.config.selected_shape,
            position: point,
        });
    }
}

impl<R: plot::Renderer> plot::Items<f64, R> for TestingEnvironment {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &Theme) {
        for item in &self.drawn_shapes {
            let size = match self.config.shape_size_unit {
                UnitType::Screen => Length::Screen(self.config.shape_size),
                UnitType::Plot => Length::Plot(self.config.shape_size as f64),
            };

            let stroke_width = match self.config.stroke_width_unit {
                UnitType::Screen => Length::Screen(self.config.stroke_width),
                UnitType::Plot => Length::Plot(self.config.stroke_width as f64),
            };

            let stroke = Stroke::new(self.config.stroke_color, stroke_width);

            match item.shape_type {
                ShapeType::Rectangle => {
                    let mut rect = shape::Rectangle::new(item.position, size, size).stroke(stroke);
                    if self.config.fill_enabled {
                        rect = rect.fill(self.config.fill_color);
                    }
                    plot.add_shape(rect);
                }
                ShapeType::Circle => {
                    let mut circ = shape::Circle::new(item.position, size).stroke(stroke);
                    if self.config.fill_enabled {
                        circ = circ.fill(self.config.fill_color);
                    }
                    plot.add_shape(circ);
                }
                ShapeType::Triangle => {
                    let mut tri = shape::Triangle::equilateral(item.position, size).stroke(stroke);
                    if self.config.fill_enabled {
                        tri = tri.fill(self.config.fill_color);
                    }
                    plot.add_shape(tri);
                }
                ShapeType::Sector => {
                    let mut sec = shape::Arc::new(
                        item.position,
                        size,
                        self.config.sector_start,
                        self.config.sector_end,
                    )
                    .stroke(stroke);

                    if self.config.sector_inner > 0.0 {
                        let percent = self.config.sector_inner as f64 / 100.0;
                        let inner = match size {
                            Length::Screen(px) => Length::Screen(px * percent as f32),
                            Length::Plot(u) => Length::Plot(u * percent),
                        };
                        sec = sec.inner_radius(inner);
                    }

                    if self.config.fill_enabled {
                        sec = sec.fill(self.config.fill_color);
                    }
                    plot.add_shape(sec);
                }
                ShapeType::Polygon => {
                    // Generates a "Strange" Irregular Polygon anchored at the click point
                    // Scale is derived from the "Shape Size" slider
                    let scale = match self.config.shape_size_unit {
                        UnitType::Plot => self.config.shape_size as f64,
                        UnitType::Screen => self.config.shape_size as f64 * 0.1, // Heuristic scale
                    };

                    // An irregular shape with a reflex angle (dent) to test offset logic
                    let points = vec![
                        PlotPoint::new(item.position.x, item.position.y), // Anchor
                        PlotPoint::new(
                            item.position.x + scale * 2.0,
                            item.position.y + scale * 0.5,
                        ),
                        PlotPoint::new(
                            item.position.x + scale * 1.0,
                            item.position.y + scale * 3.0,
                        ),
                        PlotPoint::new(
                            item.position.x - scale * 1.5,
                            item.position.y + scale * 1.0,
                        ),
                        PlotPoint::new(
                            item.position.x - scale * 0.5,
                            item.position.y + scale * 0.5,
                        ), // The Dent
                    ];
                    let mut poly = shape::Polygon::new(points).stroke(stroke);
                    if self.config.fill_enabled {
                        poly = poly.fill(self.config.fill_color);
                    }
                    plot.add_shape(poly);
                }
            }
        }
    }
}
