use iced::{
    Element, Length, Theme, keyboard, mouse,
    widget::{column, container, text},
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, TickResult},
    plot::{self, Plot, PlotData},
    scale::Linear,
    shape::{Ellipse, Polyline},
};

// -----------------------------------------------------------------------------
// 1. Application Entry
// -----------------------------------------------------------------------------
pub fn main() -> iced::Result {
    iced::application(
        ChartPlayground::new,
        ChartPlayground::update,
        ChartPlayground::view,
    )
    .title("Aksel Playground")
    .theme(Theme::Dark)
    .antialiasing(true)
    .run()
}

// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------
struct ChartPlayground {
    chart_state: State<&'static str, f64>,
    data: MyData, // <--- Your custom data struct

    // Interaction state
    cursor: Option<PlotPoint<f64>>,
}

#[derive(Debug, Clone)]
enum Message {
    Scrolled(iced::Point, mouse::ScrollDelta),
    Dragged(plot::DragDelta),
    Hovered(iced::Point),
    Unhovered,
}

impl ChartPlayground {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        // -- Setup Axes --
        // Customizable: Change Linear to Logarithmic or adjust ranges here.
        let x_scale = Linear::new(0.0, 10.0);
        let y_scale = Linear::new(-1.5, 1.5);

        let x_axis = Axis::new(x_scale, axis::Position::Bottom).skip_overlapping_labels(6.);
        let y_axis = Axis::new(y_scale, axis::Position::Left)
            // Example: Custom Tick Renderer
            .with_tick_renderer(|ctx| {
                if ctx.tick.level == 0 {
                    // Major ticks only
                    TickResult {
                        label: Some(ctx.label(format!("{:.1}", ctx.tick.value))),
                        tick_line: Some(ctx.tickline()),
                        ..Default::default()
                    }
                } else {
                    TickResult::default()
                }
            });

        state.set_axis(Self::X, x_axis);
        state.set_axis(Self::Y, y_axis);

        (
            Self {
                chart_state: state,
                data: MyData::generate(),
                cursor: None,
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            // Standard Zoom Logic (Scroll)
            Message::Scrolled(cursor_norm, delta) => {
                let y_delta = match delta {
                    mouse::ScrollDelta::Lines { y, .. } | mouse::ScrollDelta::Pixels { y, .. } => y,
                };
                let factor = if y_delta > 0.0 { 1.1 } else { 0.9 };

                // Zoom both axes based on cursor position
                if let Some(axis) = self.chart_state.axis_mut_opt(&Self::X) {
                    axis.zoom(factor, Some(cursor_norm.x));
                }
                if let Some(axis) = self.chart_state.axis_mut_opt(&Self::Y) {
                    axis.zoom(factor, Some(cursor_norm.y));
                }
            }
            // Standard Pan Logic (Drag)
            Message::Dragged(delta) => {
                self.chart_state
                    .pan_axes(Self::X, Self::Y, delta.x, delta.y);
            }
            // Hover Logic
            Message::Hovered(cursor_norm) => {
                // Convert screen coordinates (0.0-1.0) back to data coordinates
                if let (Some(x_ax), Some(y_ax)) = (
                    self.chart_state.axis_opt(&Self::X),
                    self.chart_state.axis_opt(&Self::Y),
                ) {
                    if let (Some(x), Some(y)) = (
                        x_ax.denormalize_opt(cursor_norm.x),
                        y_ax.denormalize_opt(1.0 - cursor_norm.y), // Invert Y
                    ) {
                        self.cursor = Some(PlotPoint::new(x, y));
                    }
                }
            }
            Message::Unhovered => self.cursor = None,
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Simple header showing cursor position
        let header = text(
            self.cursor
                .map(|p| format!("X: {:.2}, Y: {:.2}", p.x, p.y))
                .unwrap_or_else(|| "Hover chart to inspect".to_string()),
        );

        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, Self::X, Self::Y)
            .on_scroll(Message::Scrolled)
            .on_drag(Message::Dragged)
            .on_hover(Message::Hovered)
            .on_error(|_| Message::Unhovered);

        column![
            container(header).padding(10),
            container(chart)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(style_chart_container)
        ]
        .padding(20)
        .into()
    }
}

// -----------------------------------------------------------------------------
// 3. Data & Drawing Logic
// -----------------------------------------------------------------------------
struct MyData {
    points: Vec<PlotPoint<f64>>,
}

impl MyData {
    fn generate() -> Self {
        let points = (0..100)
            .map(|i| {
                let x = i as f64 / 10.0;
                let y = x.sin();
                PlotPoint::new(x, y)
            })
            .collect();
        Self { points }
    }
}

// This is where the magic happens.
// Use the `plot` argument to add shapes (Lines, Rectangles, Splines, etc.)
impl PlotData<f64> for MyData {
    fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
        // 1. Draw the line
        plot.add_shape(
            Polyline::new(self.points.clone())
                .stroke(Stroke::new(theme.palette().primary, Measure::Screen(2.0))),
        );

        // 2. Draw markers on points > 0.5
        for point in self.points.iter().filter(|p| p.y > 0.5) {
            plot.add_shape(
                Ellipse::circle(*point, Measure::Screen(3.0)).fill(theme.palette().danger),
            );
        }
    }
}

fn style_chart_container(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(theme.extended_palette().background.weak.color)
        .border(iced::border::rounded(8))
}
