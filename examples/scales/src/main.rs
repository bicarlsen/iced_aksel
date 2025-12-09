use aksel::{
    PlotPoint,
    scale::{Linear, Logarithmic},
};
use iced::{
    Color, Element, Length, Theme,
    alignment::{Horizontal, Vertical},
    widget::{column, container, row, text},
};
use iced_aksel::{
    Axis, Chart, State,
    axis::{self, GridLine, TickLine},
    plot::{Items, Plot},
};

// Import shapes
use iced_aksel::Stroke;
use iced_aksel::shape::{Circle, Polygon, Polyline};

// Defines the IDs for our charts' axes
type AxisId = &'static str;

pub fn main() -> iced::Result {
    iced::application(
        ScalesGallery::new,
        ScalesGallery::update,
        ScalesGallery::view,
    )
    .title("Scales & Formatting Gallery")
    .antialiasing(true)
    .run()
}

pub struct ScalesGallery {
    // Chart States
    log_state: State<AxisId, f64>,
    time_state: State<AxisId, f64>,

    // Pre-calculated Data Layers
    log_data: LogDataLayer,
    time_data: TimeDataLayer,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl ScalesGallery {
    // Axis IDs for Log Chart
    const LOG_X: &'static str = "LogX";
    const LOG_Y: &'static str = "LogY";

    // Axis IDs for Time Chart
    const TIME_X: &'static str = "TimeX";
    const TIME_Y: &'static str = "TimeY";

    pub fn new() -> (Self, iced::Task<Message>) {
        // --- 1. Setup Logarithmic Chart State ---
        let mut log_state = State::new();

        log_state.set_axis(
            Self::LOG_X,
            Axis::new(Linear::new(0.0, 10.0), axis::Position::Bottom)
                .with_tick_renderer(|ctx| Some(TickLine::simple(format!("{:.0}", ctx.tick.value))))
                .skip_overlapping_labels(6.0),
        );

        log_state.set_axis(
            Self::LOG_Y,
            Axis::new(Logarithmic::new(10.0, 1.0, 10000.0), axis::Position::Left)
                .with_tick_renderer(|ctx| {
                    let val = ctx.tick.value;
                    let label = if val >= 1000.0 {
                        format!("{:.0}k", val / 1000.0)
                    } else {
                        format!("{:.0}", val)
                    };
                    Some(TickLine::simple(label))
                })
                .with_grid_renderer(|_| {
                    Some(GridLine {
                        thickness: 1.0.into(),
                    })
                })
                .skip_overlapping_labels(6.0),
        );

        // --- 2. Setup Time/Custom Chart State ---
        let mut time_state = State::new();

        time_state.set_axis(
            Self::TIME_X,
            Axis::new(Linear::new(0.0, 12.0), axis::Position::Bottom)
                .with_tick_renderer(|ctx| {
                    let hour_offset = ctx.tick.value;
                    let hour = (12.0 + hour_offset) % 12.0;
                    let hour = if hour < 1.0 { 12.0 } else { hour };
                    let suffix = if (12.0 + hour_offset) < 24.0 {
                        "PM"
                    } else {
                        "AM"
                    };
                    Some(TickLine::simple(format!("{:.0}:00 {}", hour, suffix)))
                })
                .skip_overlapping_labels(6.0),
        );

        time_state.set_axis(
            Self::TIME_Y,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Left)
                .with_tick_renderer(|ctx| {
                    Some(TickLine::simple(format!("{:.0} %", ctx.tick.value)))
                })
                .skip_overlapping_labels(6.0),
        );

        (
            Self {
                log_state,
                time_state,
                // Initialize data once on startup
                log_data: LogDataLayer::new(),
                time_data: TimeDataLayer::new(),
            },
            iced::Task::none(),
        )
    }

    pub fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<Message> {
        column![self.view_log_chart(), self.view_time_chart()]
            .spacing(20)
            .padding(20)
            .into()
    }

    fn view_log_chart(&self) -> Element<Message> {
        // Pass the reference to the pre-calculated layer stored in struct
        let chart = Chart::new(&self.log_state).layer(&self.log_data, Self::LOG_X, Self::LOG_Y);

        column![
            text("Logarithmic Scale (Base 10)").size(16),
            text("Visualizing exponential growth (10^x)")
                .size(12)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
            container(chart)
                .height(Length::Fill)
                .width(Length::Fill)
                .padding(10)
                .style(style_container)
        ]
        .height(Length::FillPortion(1))
        .spacing(10)
        .into()
    }

    fn view_time_chart(&self) -> Element<Message> {
        // Pass the reference to the pre-calculated layer stored in struct
        let chart = Chart::new(&self.time_state).layer(&self.time_data, Self::TIME_X, Self::TIME_Y);

        column![
            text("Custom Tick Formatting").size(16),
            text("Mapping float values (0.0-12.0) to Time Strings")
                .size(12)
                .color(Color::from_rgb(0.5, 0.5, 0.5)),
            container(chart)
                .height(Length::Fill)
                .width(Length::Fill)
                .padding(10)
                .style(style_container)
        ]
        .height(Length::FillPortion(1))
        .spacing(10)
        .into()
    }
}

// --- Style Helper ---
fn style_container(theme: &Theme) -> container::Style {
    container::Style::default().border(iced::border::color(theme.palette().text).width(1.0))
}

// =========================================================
//  DATA LAYERS (The "Items" Implementations)
// =========================================================

// --- 1. Exponential Data (For Log Chart) ---
struct LogDataLayer {
    line_points: Vec<PlotPoint<f64>>,
    marker_points: Vec<PlotPoint<f64>>,
}

impl LogDataLayer {
    fn new() -> Self {
        // Generate Exponential Data: y = 10^(x/2)
        let line_points = (0..=100)
            .map(|i| {
                let x = i as f64 / 10.0;
                let y = 10.0f64.powf(x / 2.5);
                PlotPoint::new(x, y)
            })
            .collect();

        // Specific markers at integer powers
        let marker_points = (0..=4)
            .map(|i| {
                let x = i as f64 * 2.5;
                let y = 10.0f64.powf(i as f64);
                PlotPoint::new(x, y)
            })
            .collect();

        Self {
            line_points,
            marker_points,
        }
    }
}

impl Items<f64, iced::Renderer, Theme> for LogDataLayer {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        // Use pre-calculated points. Cloning the Vec is cheap (it's just a ref counter or simple copy of struct data if owned by Shape)
        // Note: Polyline::new usually takes ownership or clones internally.
        plot.add_shape(Polyline::new(
            self.line_points.clone(),
            Stroke::new(theme.palette().primary, iced_aksel::Length::Screen(2.5)),
        ));

        for point in &self.marker_points {
            plot.add_shape(
                Circle::new(*point, iced_aksel::Length::Screen(6.0)).fill(theme.palette().danger),
            );
        }
    }
}

// --- 2. Time Series Data (For Custom Chart) ---
struct TimeDataLayer {
    points: Vec<PlotPoint<f64>>,
    poly_points: Vec<PlotPoint<f64>>,
}

impl TimeDataLayer {
    fn new() -> Self {
        // Generate Sine Wave Data
        let points: Vec<PlotPoint<f64>> = (0..=120)
            .map(|i| {
                let x = i as f64 / 10.0;
                let y = 50.0 + (x * 1.5).sin() * 30.0 + (x * 4.0).cos() * 10.0;
                PlotPoint::new(x, y)
            })
            .collect();

        // Close polygon for the filled area
        let mut poly_points = points.clone();
        if let (Some(first), Some(last)) = (points.first(), points.last()) {
            poly_points.push(PlotPoint::new(last.x, 0.0));
            poly_points.push(PlotPoint::new(first.x, 0.0));
        }

        Self {
            points,
            poly_points,
        }
    }
}

impl Items<f64, iced::Renderer, Theme> for TimeDataLayer {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        plot.add_shape(Polygon::new(self.poly_points.clone()).fill(Color {
            a: 0.3,
            ..theme.palette().success
        }));

        plot.add_shape(Polyline::new(
            self.points.clone(),
            Stroke::new(theme.palette().success, iced_aksel::Length::Screen(2.0)),
        ));
    }
}
