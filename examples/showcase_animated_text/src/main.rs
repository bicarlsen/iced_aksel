use iced::alignment::{Horizontal, Vertical};
use iced::{
    Color, Element, Length, Subscription, Theme,
    widget::{column, container},
    window,
};
use iced_aksel::{
    Axis, Chart, Measure, PlotPoint, Quality, State, Stroke,
    axis::{self},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::{Ellipse, Label, Line},
};

// -----------------------------------------------------------------------------
// 1. Application Entry
// -----------------------------------------------------------------------------
fn main() -> iced::Result {
    iced::application(
        AnimatedTextApp::new,
        AnimatedTextApp::update,
        AnimatedTextApp::view,
    )
    .subscription(AnimatedTextApp::subscription)
    .theme(Theme::Dark)
    .antialiasing(true)
    .run()
}

// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------
struct AnimatedTextApp {
    chart_state: State<&'static str, f64>,
    data: ChartData,
    frame_count: u64,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

impl AnimatedTextApp {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        // Setup axes with a nice range
        let x_scale = Linear::new(0.0, 100.0);
        let y_scale = Linear::new(0.0, 100.0);

        let x_axis = Axis::new(x_scale, axis::Position::Bottom);
        let y_axis = Axis::new(y_scale, axis::Position::Left);

        state.set_axis(Self::X, x_axis);
        state.set_axis(Self::Y, y_axis);

        (
            Self {
                chart_state: state,
                data: ChartData::new(),
                frame_count: 0,
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.chart_state.mark_dirty();
                self.frame_count += 1;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, Self::X, Self::Y)
            .quality(Quality::High);

        column![container(chart).width(Length::Fill).height(Length::Fill)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(|_| Message::Tick)
    }
}

// -----------------------------------------------------------------------------
// 3. Chart Data & Drawing Logic
// -----------------------------------------------------------------------------
struct ChartData {
    points: Vec<PlotPoint>,
}

impl ChartData {
    fn new() -> Self {
        // Generate some sample data for a nice curve
        let points: Vec<PlotPoint> = (0..=100)
            .map(|x| {
                let x_val = x as f64;
                let y_val = 50.0 + 30.0 * ((x_val / 10.0).sin());
                PlotPoint::new(x_val, y_val)
            })
            .collect();

        Self { points }
    }
}

impl PlotData<f64> for ChartData {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        // Draw the sine wave line
        for i in 0..self.points.len() - 1 {
            plot.add_shape(Line::new(
                self.points[i],
                self.points[i + 1],
                Stroke::new(Color::from_rgb(0.3, 0.7, 1.0), Measure::Screen(2.0)),
            ));
        }

        // Draw dots at the data points
        for point in &self.points {
            plot.add_shape(
                Ellipse::circle(*point, Measure::Screen(3.0)).fill(Color::from_rgb(0.2, 0.5, 0.8)),
            );
        }

        // Get the current frame count for animation
        // We'll use the plot's center point at (50, 50)
        let center = PlotPoint::new(50.0, 50.0);

        // Calculate oscillating text size using sine wave
        // The app state isn't directly accessible here, so we'll use time-based animation
        // Using a simple counter approach via the drawing cycle
        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64
            / 1000.0; // Convert to seconds

        // Use modulo to keep time in a reasonable range (0-10 seconds cycle)
        // This prevents floating-point precision issues with very large numbers
        let time = time % 10.0;

        // Oscillate between 20 and 60 pixels
        let min_size = 20.0;
        let max_size = 60.0;
        let size_range = max_size - min_size;
        let text_size = min_size + size_range * (0.5 + 0.5 * (time * 2.0).sin());

        // Draw animated text in the center
        plot.add_shape(
            Label::new("ANIMATED", center)
                .size(Measure::Screen(text_size as f32))
                .align(Horizontal::Center, Vertical::Center)
                .fill(Color::from_rgb(1.0, 0.8, 0.2)),
        );

        // Draw a subtle background circle that also animates
        let circle_radius = 30.0 + 10.0 * (time * 2.0).sin();
        plot.add_shape(
            Ellipse::circle(center, Measure::Screen(circle_radius as f32)).stroke(Stroke::new(
                Color::from_rgba(1.0, 0.8, 0.2, 0.3),
                Measure::Screen(2.0),
            )),
        );
    }
}
