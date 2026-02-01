use iced::{
    Element, Length, Subscription, Task, Theme,
    time::Instant,
    widget::{column, container, row, slider, text},
    window,
};

use iced_aksel::{
    Chart, Measure, PlotPoint, State,
    axis::{self, Axis},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Rectangle,
};

// -----------------------------------------------------------------------------
// 1. Main Entry
// -----------------------------------------------------------------------------
pub fn main() -> iced::Result {
    iced::application(PerfApp::default, PerfApp::update, PerfApp::view)
        .subscription(PerfApp::subscription)
        .theme(Theme::Dark)
        .run()
}

// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------
struct PerfApp {
    // Chart State
    chart_state: State<String, f64>,

    // Performance Data (The "Model" for our Plot)
    data: RectangleGridData,

    // FPS Counter State
    last_frame_time: Option<Instant>,
    fps: f32,
}

#[derive(Debug, Clone)]
enum Message {
    /// Updates the target rectangle count from the slider
    CountChanged(f64),
    /// Triggers on every frame to calculate FPS and force redraw
    Tick(Instant),
}

impl Default for PerfApp {
    fn default() -> Self {
        let mut state = State::new();

        // Setup axes with a fixed large range to fit our grid
        // We expect a grid of roughly 316x316 for 100k items.
        // Let's set the view to 0..500 to start.
        state.set_axis(
            "x".to_string(),
            Axis::new(Linear::new(0.0, 500.0), axis::Position::Bottom),
        );
        state.set_axis(
            "y".to_string(),
            Axis::new(Linear::new(0.0, 500.0), axis::Position::Left),
        );

        Self {
            chart_state: state,
            // Start with a modest load
            data: RectangleGridData { count: 10_000 },
            last_frame_time: None,
            fps: 0.0,
        }
    }
}

impl PerfApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CountChanged(value) => {
                self.data.count = value as usize;
                Task::none()
            }
            Message::Tick(now) => {
                // Calculate FPS using Exponential Moving Average
                if let Some(last) = self.last_frame_time {
                    let delta = now.duration_since(last);
                    let delta_secs = delta.as_secs_f32();
                    if delta_secs > 0.0 {
                        let instant_fps = 1.0 / delta_secs;
                        // Smooth the value: 90% history, 10% new
                        self.fps = self.fps * 0.9 + instant_fps * 0.1;
                    }
                }
                self.last_frame_time = Some(now);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // 1. Controls Header
        let controls =
            row![
                // FPS Counter
                container(text(format!("FPS: {:.0}", self.fps)).size(20).color(
                    if self.fps < 30.0 {
                        iced::Color::from_rgb(1.0, 0.2, 0.2)
                    } else {
                        iced::Color::WHITE
                    }
                ))
                .padding(10)
                .style(container::rounded_box),
                // Rectangle Count Slider
                column![
                    text(format!("Rectangles: {}", self.data.count)),
                    slider(
                        0.0..=100_000.0,
                        self.data.count as f64,
                        Message::CountChanged
                    )
                    .step(100.0)
                    .width(Length::Fixed(300.0))
                ]
                .spacing(5),
                text("Note: Zoom/Pan the chart to verify culling.")
                    .size(14)
                    .color(iced::Color::from_rgb(0.7, 0.7, 0.7))
            ]
            .spacing(40)
            .align_y(iced::Alignment::Center)
            .padding(20);

        // 2. Chart Area
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, "x".to_string(), "y".to_string())
            .width(Length::Fill)
            .height(Length::Fill);

        column![controls, chart].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Request a Tick every frame to stress-test the rendering pipeline
        window::frames().map(Message::Tick)
    }
}

// -----------------------------------------------------------------------------
// 3. Data Logic (Generating the shapes)
// -----------------------------------------------------------------------------

struct RectangleGridData {
    count: usize,
}

impl PlotData<f64> for RectangleGridData {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        // Arrange rectangles in a square grid
        // e.g., 10,000 rects -> 100x100 grid
        let grid_width = (self.count as f64).sqrt().ceil() as usize;

        let spacing = 2.0; // Space between rects
        let size = 1.0; // Size of each rect in plot units

        for i in 0..self.count {
            let col = i % grid_width;
            let row = i / grid_width;

            let x = col as f64 * spacing;
            let y = row as f64 * spacing;

            // Use the centered constructor for uniform sizes
            // We use Corners here to ensure they scale with the plot (Data Coordinates)
            // If you want fixed pixel size, use Rectangle::centered with Measure::Screen
            plot.add_shape(
                Rectangle::corners(PlotPoint::new(x, y), PlotPoint::new(x + size, y + size))
                    // Color based on position to look cool
                    .fill(iced::Color::from_rgb(
                        (col as f32 / grid_width as f32),
                        (row as f32 / grid_width as f32),
                        0.5,
                    )),
            );
        }
    }
}
