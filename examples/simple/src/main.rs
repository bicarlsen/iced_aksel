use aksel::{PlotPoint, scale::Linear};
use iced::{
    Color, Element, Length, Theme,
    widget::{column, container, text},
};
use iced_aksel::{
    Axis, Chart, Measure, State,
    axis::{self},
    plot::{Plot, PlotData},
    shape::Rectangle,
};
use rand::Rng;

// -----------------------------------------------------------------------------
// Application Entry
// -----------------------------------------------------------------------------

pub fn main() -> iced::Result {
    iced::application(
        SimpleExample::new,
        SimpleExample::update,
        SimpleExample::view,
    )
    .theme(Theme::Dark)
    .antialiasing(true)
    .run()
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

struct SimpleExample {
    chart_state: State<&'static str, f64>,
    solar_system: SolarSystem,
}

#[derive(Debug, Clone)]
enum Message {
    Scrolled(iced::Point, iced::mouse::ScrollDelta),
    Dragged(iced_aksel::DragDelta),
}

impl SimpleExample {
    const X_ID: &'static str = "x";
    const Y_ID: &'static str = "y";

    // Initial world bounds (-100 to 100)
    const WORLD_SIZE: f64 = 100.0;

    // Limit how far out the user can zoom (World Span > 500)
    const MAX_ZOOM_OUT_SPAN: f64 = 500.0;

    fn new() -> (Self, iced::Task<Message>) {
        let mut chart_state = State::new();

        // 1. Setup Axes
        // We set them to invisible as this is a "Space" view
        chart_state.set_axis(
            Self::X_ID,
            Axis::new(
                Linear::new(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                axis::Position::Bottom,
            )
            .invisible(),
        );

        chart_state.set_axis(
            Self::Y_ID,
            Axis::new(
                Linear::new(-Self::WORLD_SIZE, Self::WORLD_SIZE),
                axis::Position::Left,
            )
            .invisible(),
        );

        // 2. Generate Data
        // 500 Stars/Planets
        let solar_system = SolarSystem::generate(500);

        (
            Self {
                chart_state,
                solar_system,
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::Scrolled(cursor_norm, delta) => {
                let delta_y = match delta {
                    iced::mouse::ScrollDelta::Lines { y, .. }
                    | iced::mouse::ScrollDelta::Pixels { y, .. } => y,
                };

                // Zoom Factor: > 1.0 zooms in, < 1.0 zooms out
                let factor = if delta_y > 0.0 { 1.10 } else { 0.90 };

                // Constraint Logic: Check current span before applying zoom out
                let can_zoom = if factor < 1.0 {
                    // If zooming out, check if we are already too wide
                    let current_span = self
                        .chart_state
                        .axis(&Self::X_ID)
                        .map(|ax| {
                            let (min, max) = ax.domain();
                            max - min
                        })
                        .unwrap_or(0.0);

                    current_span < Self::MAX_ZOOM_OUT_SPAN
                } else {
                    true // Always allow zooming in
                };

                if can_zoom {
                    if let Some(axis) = self.chart_state.axis_mut(&Self::X_ID) {
                        axis.zoom(factor, Some(cursor_norm.x));
                    }
                    if let Some(axis) = self.chart_state.axis_mut(&Self::Y_ID) {
                        axis.zoom(factor, Some(1.0 - cursor_norm.y));
                    }
                }
            }
            Message::Dragged(delta) => {
                // Standard Pan Logic
                // We invert both axes to match the "Camera/Viewport" control feel.
                self.chart_state
                    .pan_axes(Self::X_ID, Self::Y_ID, -delta.x, -delta.y);
            }
        }
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.solar_system, Self::X_ID, Self::Y_ID)
            .on_scroll(Message::Scrolled)
            .on_drag(Message::Dragged);

        let instruction = container(
            text("Scroll to Zoom. Drag to Pan.\nNotice how background stars (white) stay fixed size, while planets (colored) scale.")
                .size(14)
                .align_x(iced::alignment::Horizontal::Center)
                .color(Color::from_rgb(0.7, 0.7, 0.7))
        )
        .padding(15)
        .width(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center);

        column![
            instruction,
            container(chart).width(Length::Fill).height(Length::Fill)
        ]
        .into()
    }
}

// -----------------------------------------------------------------------------
// Data Model
// -----------------------------------------------------------------------------

struct Star {
    position: PlotPoint,
    // Using `Measure<f64>` allows us to mix Screen pixels and Plot units
    size: Measure<f64>,
    color: Color,
}

struct SolarSystem {
    objects: Vec<Star>,
}

impl SolarSystem {
    fn generate(count: usize) -> Self {
        let mut rng = rand::rng();
        let mut objects = Vec::with_capacity(count);

        // 1. Create "Background Stars"
        // Use Measure::Screen: These stay the same pixel size regardless of zoom
        // giving the illusion of being very far away.
        for _ in 0..(count * 9 / 10) {
            objects.push(Star {
                position: PlotPoint {
                    x: rng.random_range(-200.0..200.0),
                    y: rng.random_range(-200.0..200.0),
                },
                size: Measure::Screen(rng.random_range(1.5..3.0)),
                color: Color::from_rgb(
                    rng.random_range(0.8..1.0),
                    rng.random_range(0.8..1.0),
                    rng.random_range(0.9..1.0),
                ),
            });
        }

        // 2. Create "Planets"
        // Use Measure::Plot: These sizes are in Data Units.
        // They will enlarge when you zoom in, behaving like local objects.
        for _ in 0..(count / 10) {
            objects.push(Star {
                position: PlotPoint {
                    x: rng.random_range(-80.0..80.0),
                    y: rng.random_range(-80.0..80.0),
                },
                size: Measure::Plot(rng.random_range(3.0..8.0)),
                color: Color::from_rgb(
                    rng.random_range(0.8..1.0),
                    rng.random_range(0.4..0.6),
                    rng.random_range(0.4..0.6),
                ),
            });
        }

        Self { objects }
    }
}

impl PlotData<f64, iced::Renderer, Theme> for SolarSystem {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, _theme: &Theme) {
        for object in &self.objects {
            // Draw object as a square
            let shape =
                Rectangle::new(object.position, object.size, object.size).fill(object.color);

            plot.add_shape(shape);
        }
    }
}
