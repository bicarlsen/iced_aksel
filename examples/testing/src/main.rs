use iced::{
    Element, Length, Subscription, Task, Theme,
    time::Instant,
    widget::{column, container, row, text},
    window,
};

use iced_aksel::stroke::StrokeStyle;
use iced_aksel::{
    Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, Axis},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Rectangle,
};

// -----------------------------------------------------------------------------
// 1. Main Entry
// -----------------------------------------------------------------------------
pub fn main() -> iced::Result {
    iced::application(GalleryApp::default, GalleryApp::update, GalleryApp::view)
        .subscription(GalleryApp::subscription)
        .theme(Theme::Dark)
        .run()
}

// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------

struct GalleryApp {
    // Chart State
    chart_state: State<String, f64>,

    // The Model
    data: RectangleGallery,

    // FPS Counter State
    last_frame_time: Option<Instant>,
    fps: f32,
}

#[derive(Debug, Clone)]
enum Message {
    /// Triggers on every frame
    Tick(Instant),
}

impl Default for GalleryApp {
    fn default() -> Self {
        let mut state = State::new();

        // Setup axes to fit our 5x5 gallery comfortably
        state.set_axis(
            "x".to_string(),
            Axis::new(Linear::new(0.0, 60.0), axis::Position::Bottom),
        );
        state.set_axis(
            "y".to_string(),
            Axis::new(Linear::new(0.0, 60.0), axis::Position::Left),
        );

        Self {
            chart_state: state,
            data: RectangleGallery,
            last_frame_time: None,
            fps: 0.0,
        }
    }
}

impl GalleryApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                // Calculate FPS
                if let Some(last) = self.last_frame_time {
                    let delta = now.duration_since(last);
                    let delta_secs = delta.as_secs_f32();
                    if delta_secs > 0.0 {
                        let instant_fps = 1.0 / delta_secs;
                        self.fps = self.fps * 0.9 + instant_fps * 0.1;
                    }
                }
                self.last_frame_time = Some(now);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // 1. Header
        let header = row![
            container(
                text(format!("FPS: {:.0}", self.fps))
                    .size(20)
                    .color(if self.fps < 30.0 { iced::Color::from_rgb(1.0, 0.2, 0.2) } else { iced::Color::WHITE })
            )
            .padding(10)
            .style(container::rounded_box),

            column![
                text("Rectangle Gallery").size(18),
                text("Row 1: Fill (Corners) | Row 2: Stroke (Corners) | Row 3: Centered (Fixed Px) | Row 4: Centered (Plot Units)")
                    .size(14)
                    .color(iced::Color::from_rgb(0.7, 0.7, 0.7)),
            ].spacing(5)
        ]
            .spacing(20)
            .padding(20)
            .align_y(iced::Alignment::Center);

        // 2. Chart Area
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, "x".to_string(), "y".to_string())
            .width(Length::Fill)
            .height(Length::Fill);

        column![header, chart].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }
}

// -----------------------------------------------------------------------------
// 3. Data Logic (The Gallery)
// -----------------------------------------------------------------------------

struct RectangleGallery;

impl PlotData<f64> for RectangleGallery {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        // We will draw a 5x5 grid (roughly) where each row showcases a feature.
        // X spacing = 10 units, Y spacing = 10 units.

        let cols = 5;

        // --- ROW 0: Basic Fills (Defined by Corners) ---
        // Tests: Geometry::Corners + Fill
        let y = 50.0;
        for i in 0..cols {
            let x = 5.0 + (i as f64 * 10.0);

            plot.add_shape(
                Rectangle::corners(PlotPoint::new(x, y), PlotPoint::new(x + 5.0, y + 5.0))
                    // Varying opacity and colors
                    .fill(iced::Color::from_rgba(
                        0.2,
                        0.6,
                        1.0,
                        1.0 - (i as f32 * 0.2),
                    )),
            );
        }

        // --- ROW 1: Basic Strokes (Defined by Corners) ---
        // Tests: Geometry::Corners + Stroke (Screen Width)
        let y = 40.0;
        for i in 0..cols {
            let x = 5.0 + (i as f64 * 10.0);
            let thickness = 1.0 + (i as f32 * 2.0); // 1px, 3px, 5px...

            plot.add_shape(
                Rectangle::corners(PlotPoint::new(x, y), PlotPoint::new(x + 5.0, y + 5.0)).stroke(
                    Stroke::new(
                        iced::Color::from_rgb(1.0, 0.8, 0.2),
                        Measure::Screen(thickness),
                    ),
                ),
            );
        }

        // --- ROW 2: Fixed Pixel Size (Centered) ---
        // Tests: Geometry::Centered + Measure::Screen
        // These should NOT change size when you zoom the chart.
        let y = 30.0;
        for i in 0..cols {
            let x = 7.5 + (i as f64 * 10.0); // Center point
            let size = 10.0 + (i as f32 * 5.0); // 10px, 15px, 20px...

            plot.add_shape(
                Rectangle::centered(
                    PlotPoint::new(x, y + 2.5), // Center roughly in the "cell"
                    Measure::Screen(size),
                    Measure::Screen(size),
                )
                .fill(iced::Color::from_rgb(0.8, 0.2, 0.8))
                // Add a thin white border to prove stroke+fill works
                .stroke(Stroke::new(iced::Color::WHITE, Measure::Screen(1.0))),
            );
        }

        // --- ROW 3: Scalable Plot Size (Centered) ---
        // Tests: Geometry::Centered + Measure::Plot
        // These SHOULD zoom with the chart.
        let y = 20.0;
        for i in 0..cols {
            let x = 7.5 + (i as f64 * 10.0);
            // Width/Height in Data Units
            let w = 1.0 + (i as f64 * 1.5);
            let h = 4.0 - (i as f64 * 0.5);

            plot.add_shape(
                Rectangle::centered(
                    PlotPoint::new(x, y + 2.5),
                    Measure::Plot(w),
                    Measure::Plot(h),
                )
                .stroke(Stroke::with_style(
                    iced::Color::from_rgb(0.2, 0.8, 0.2),
                    Measure::Screen(2.0),
                    StrokeStyle::Dashed { dash: 5., gap: 5. },
                )),
            );
        }

        // --- ROW 4: Complex (Transparent Fill + Thick Stroke) ---
        // Tests: Alpha Blending & Stroke Alignment
        let y = 10.0;
        for i in 0..cols {
            let x = 5.0 + (i as f64 * 10.0);

            plot.add_shape(
                Rectangle::corners(PlotPoint::new(x, y), PlotPoint::new(x + 5.0, y + 5.0))
                    .fill(iced::Color::from_rgba(1.0, 0.0, 0.0, 0.3)) // Semi-transparent red
                    .stroke(Stroke::new(
                        iced::Color::from_rgb(1.0, 1.0, 1.0),
                        Measure::Screen(2.0 + i as f32),
                    )),
            );
        }
    }
}
