use aksel::{
    PlotPoint,
    scale::{Linear, Logarithmic},
};
use iced::{
    Color, Length, Theme,
    alignment::Horizontal,
    widget::{column, container, row, text},
};
use iced_aksel::Stroke;
use iced_aksel::shape::{Circle, Polyline};
use iced_aksel::{
    Axis, Chart, State,
    axis::{self, GridLine, TickLine},
    plot::{Items, Plot},
};

// -----------------------------------------------------------------------------
// Application Entry
// -----------------------------------------------------------------------------

pub fn main() -> iced::Result {
    iced::application(
        ScalesExample::new,
        ScalesExample::update,
        ScalesExample::view,
    )
    .title("Comparison: Linear vs Logarithmic")
    .antialiasing(true)
    .run()
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

pub struct ScalesExample {
    // We use two chart states to view the SAME data differently
    linear_view: State<&'static str, f64>,
    log_view: State<&'static str, f64>,

    // The data model (shared between both charts)
    data: ExponentialData,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl ScalesExample {
    const AXIS_X: &'static str = "x";
    const AXIS_Y: &'static str = "y";

    pub fn new() -> (Self, iced::Task<Message>) {
        // Range: 0 to 5 on X, 1 to 100,000 on Y
        let x_min = 0.0;
        let x_max = 5.0;
        let y_min = 1.0;
        let y_max = 100_000.0;

        // --- Chart 1: The Linear View ---
        // This will look like a sharp vertical wall
        let mut linear_view = State::new();
        linear_view.set_axis(
            Self::AXIS_X,
            Axis::new(Linear::new(x_min, x_max), axis::Position::Bottom)
                .with_tick_renderer(|ctx| {
                    // Only show Major ticks
                    if ctx.tick.level != 0 {
                        return None;
                    }
                    Some(TickLine::simple(format!("{:.0}", ctx.tick.value)))
                })
                .skip_overlapping_labels(6.0),
        );
        linear_view.set_axis(
            Self::AXIS_Y,
            Axis::new(Linear::new(y_min, y_max), axis::Position::Left)
                .with_tick_renderer(|ctx| {
                    // Only show Major ticks
                    if ctx.tick.level != 0 {
                        return None;
                    }
                    let val = ctx.tick.value;
                    if val >= 1000.0 {
                        Some(TickLine::simple(format!("{:.0}k", val / 1000.0)))
                    } else {
                        Some(TickLine::simple(format!("{:.0}", val)))
                    }
                })
                .with_grid_renderer(|_| {
                    Some(GridLine {
                        thickness: 1.0.into(),
                    })
                })
                .skip_overlapping_labels(6.0),
        );

        // --- Chart 2: The Logarithmic View ---
        // This will look like a straight line (Log(10^x) = x * Log(10))
        let mut log_view = State::new();
        log_view.set_axis(
            Self::AXIS_X,
            Axis::new(Linear::new(x_min, x_max), axis::Position::Bottom)
                .with_tick_renderer(|ctx| {
                    // Only show Major ticks
                    if ctx.tick.level != 0 {
                        return None;
                    }
                    Some(TickLine::simple(format!("{:.0}", ctx.tick.value)))
                })
                .skip_overlapping_labels(6.0),
        );
        log_view.set_axis(
            Self::AXIS_Y,
            // Notice: Logarithmic scale here
            Axis::new(Logarithmic::new(10.0, y_min, y_max), axis::Position::Left)
                .with_tick_renderer(|ctx| {
                    // Only show Major ticks
                    if ctx.tick.level != 0 {
                        return None;
                    }
                    let val = ctx.tick.value;
                    if val >= 1000.0 {
                        Some(TickLine::simple(format!("{:.0}k", val / 1000.0)))
                    } else {
                        Some(TickLine::simple(format!("{:.0}", val)))
                    }
                })
                .with_grid_renderer(|_| {
                    Some(GridLine {
                        thickness: 1.0.into(),
                    })
                })
                .skip_overlapping_labels(6.0),
        );

        (
            Self {
                linear_view,
                log_view,
                data: ExponentialData::new(),
            },
            iced::Task::none(),
        )
    }

    pub fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
        column![
            // Row of Charts
            row![
                self.view_linear_chart(),
                self.view_log_chart()
            ]
            .spacing(20)
            .height(Length::Fill),

            // Footer Information
            container(
                text("Scales are fully customizable. You can even implement your own scale via the `aksel::scale::Scale` trait.")
                    .size(14)
                    .color(Color::from_rgb(0.5, 0.5, 0.5))
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center)
        ]
        .spacing(20)
        .padding(20)
        .into()
    }

    fn view_linear_chart(&self) -> iced::Element<'_, Message> {
        let chart = Chart::new(&self.linear_view).layer(&self.data, Self::AXIS_X, Self::AXIS_Y);

        column![
            text("Linear Scale").size(20),
            text("Same data. Notice how small values are squashed at the bottom.")
                .size(14)
                .color(Color::from_rgb(0.4, 0.4, 0.4)),
            container(chart)
                .height(Length::Fill)
                .width(Length::Fill)
                .padding(10)
                .style(style_container)
        ]
        .width(Length::FillPortion(1))
        .spacing(10)
        .into()
    }

    fn view_log_chart(&self) -> iced::Element<'_, Message> {
        let chart = Chart::new(&self.log_view).layer(&self.data, Self::AXIS_X, Self::AXIS_Y);

        column![
            text("Logarithmic Scale").size(20),
            text("Same data. Constant growth rate appears as a straight line.")
                .size(14)
                .color(Color::from_rgb(0.4, 0.4, 0.4)),
            container(chart)
                .height(Length::Fill)
                .width(Length::Fill)
                .padding(10)
                .style(style_container)
        ]
        .width(Length::FillPortion(1))
        .spacing(10)
        .into()
    }
}

// -----------------------------------------------------------------------------
// Data Layer
// -----------------------------------------------------------------------------

struct ExponentialData {
    line: Vec<PlotPoint<f64>>,
    markers: Vec<PlotPoint<f64>>,
}

impl ExponentialData {
    fn new() -> Self {
        // Simple Math: y = 10^x
        // We plot points from x=0 to x=5
        // x=0 -> y=1
        // x=1 -> y=10
        // ...
        // x=5 -> y=100,000

        let line = (0..=50)
            .map(|i| {
                let x = i as f64 / 10.0;
                let y = 10.0f64.powf(x);
                PlotPoint::new(x, y)
            })
            .collect();

        // Markers at integer steps to make reading the grid easier
        let markers = (0..=5)
            .map(|i| {
                let x = i as f64;
                let y = 10.0f64.powf(x);
                PlotPoint::new(x, y)
            })
            .collect();

        Self { line, markers }
    }
}

impl Items<f64, iced::Renderer, Theme> for ExponentialData {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        // Draw the line
        plot.add_shape(Polyline::new(
            self.line.clone(),
            Stroke::new(theme.palette().primary, iced_aksel::Length::Screen(2.5)),
        ));

        // Draw dots at 1, 10, 100, 1000...
        for point in &self.markers {
            plot.add_shape(
                Circle::new(*point, iced_aksel::Length::Screen(5.0)).fill(theme.palette().danger),
            );
        }
    }
}

// -----------------------------------------------------------------------------
// Styles
// -----------------------------------------------------------------------------

fn style_container(theme: &Theme) -> container::Style {
    container::Style::default().border(iced::border::color(theme.palette().text).width(1.0))
}
