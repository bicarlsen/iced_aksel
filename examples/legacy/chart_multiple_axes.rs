//! Multi-axis chart example
//!
//! This example demonstrates how to use the `Chart` widget with multiple axes and scales.
//! It features:
//! - One shared X-axis (Bottom)
//! - Two independent Y-axes (Left and Right)
//! - Two data series (Sine and Cosine waves) mapped to different Y-axes
//! - Interactive features: Panning, Axis Dragging, and Hover Labels

use iced::{
    Color, Element, Point, Task, Theme,
    mouse::ScrollDelta,
    widget::{Slider, column, row, text},
};

use aksel::{PlotPoint, PlotRect, scale::Linear};
use iced_extras::widget::chart::{
    Axis, Chart, DragDelta, Layer, Position, State,
    axis::{self, TickLine},
    shape::{self, Label, Length},
};

// --- Axis Identifiers ---
const AXIS_X: &str = "axis_x";
const AXIS_Y_LEFT: &str = "axis_y_left";
const AXIS_Y_RIGHT: &str = "axis_y_right";

// --- Constants ---
const PLOT_BOUNDS_X: (f64, f64) = (0.0, 10.0);
const PLOT_BOUNDS_Y_LEFT: (f64, f64) = (-1.5, 1.5);
const PLOT_BOUNDS_Y_RIGHT: (f64, f64) = (-5.0, 5.0); // Different scale for the second axis

type AxisId = &'static str;

#[derive(Debug, Clone)]
enum Message {
    PointCountChanged(u32),
    PanChart(DragDelta),
    AxisDragged(AxisId, f32),
    AxisHovered(AxisId, f32),
    ChartScrolled(Point, ScrollDelta),
    AxisScrolled(AxisId, f32, ScrollDelta),
}

struct ExampleApp {
    state: State<AxisId, f64>,
    layers: Vec<Layer<AxisId, f64>>,
    point_count: u32,
    hovered_axis_label: Option<(AxisId, String, Point)>, // Store label info for rendering overlay if needed, or just print for now
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut state = State::new();

        // --- 1. Setup Axes ---

        // X-Axis (Bottom)
        let axis_x = Axis::new(
            Linear::new(PLOT_BOUNDS_X.0, PLOT_BOUNDS_X.1),
            Position::Bottom,
        )
        .with_tick_renderer(|tick| {
            Some(TickLine {
                thickness: iced::Pixels(1.0),
                color: Color::WHITE,
                label: Some(axis::Label {
                    color: Color::WHITE,
                    size: iced::Pixels(10.0),
                    content: format!("{:.1}", tick.value),
                }),
                length: iced::Pixels(5.0),
            })
        });

        // Y-Axis Left (for Sine wave)
        let axis_y_left = Axis::new(
            Linear::new(PLOT_BOUNDS_Y_LEFT.0, PLOT_BOUNDS_Y_LEFT.1),
            Position::Left,
        )
        .with_tick_renderer(|tick| {
            Some(TickLine {
                thickness: iced::Pixels(1.0),
                color: Color::from_rgb(0.4, 0.7, 1.0), // Blue-ish
                label: Some(axis::Label {
                    color: Color::from_rgb(0.4, 0.7, 1.0),
                    size: iced::Pixels(10.0),
                    content: format!("{:.1}", tick.value),
                }),
                length: iced::Pixels(5.0),
            })
        });

        // Y-Axis Right (for Cosine wave, different scale)
        let axis_y_right = Axis::new(
            Linear::new(PLOT_BOUNDS_Y_RIGHT.0, PLOT_BOUNDS_Y_RIGHT.1),
            Position::Right,
        )
        .with_tick_renderer(|tick| {
            Some(TickLine {
                thickness: iced::Pixels(1.0),
                color: Color::from_rgb(1.0, 0.6, 0.2), // Orange-ish
                label: Some(axis::Label {
                    color: Color::from_rgb(1.0, 0.6, 0.2),
                    size: iced::Pixels(10.0),
                    content: format!("{:.1}", tick.value),
                }),
                length: iced::Pixels(5.0),
            })
        });

        state.set_axis(AXIS_X, axis_x);
        state.set_axis(AXIS_Y_LEFT, axis_y_left);
        state.set_axis(AXIS_Y_RIGHT, axis_y_right);

        // --- 2. Initial Data Generation ---
        let point_count = 100;
        let layers = generate_layers(point_count);

        (
            Self {
                state,
                layers,
                point_count,
                hovered_axis_label: None,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PointCountChanged(count) => {
                self.point_count = count;
                self.layers = generate_layers(count);
            }
            Message::PanChart(delta) => {
                // Pan X axis
                self.state
                    .get_axis_mut(&AXIS_X)
                    .unwrap()
                    .scale_mut()
                    .pan(delta.x as f64);

                // Pan both Y axes
                self.state
                    .get_axis_mut(&AXIS_Y_LEFT)
                    .unwrap()
                    .scale_mut()
                    .pan(delta.y as f64);
                self.state
                    .get_axis_mut(&AXIS_Y_RIGHT)
                    .unwrap()
                    .scale_mut()
                    .pan(delta.y as f64);
            }
            Message::AxisDragged(axis_id, delta) => {
                if let Some(axis) = self.state.get_axis_mut(&axis_id) {
                    // Zoom/Scale the axis based on drag
                    // For simplicity in this example, we'll just pan it,
                    // but you could implement zoom logic here.
                    axis.scale_mut().pan(delta as f64);
                }
            }
            Message::AxisHovered(axis_id, _normalized_value) => {
                // In a real app, you might update a tooltip here.
                // For now, we rely on the Chart's internal cursor_formatter if set,
                // or we could store this to render a custom overlay.
                // The Chart widget's `cursor_formatter` handles the immediate visual feedback on the axis.
                // This message is useful if we want to trigger other UI updates.
                // println!("Hovered axis: {}, val: {}", axis_id, normalized_value);
            }
            Message::ChartScrolled(point, delta) => {
                let y = match delta {
                    ScrollDelta::Lines { y, .. } => y,
                    ScrollDelta::Pixels { y, .. } => y,
                };
                let zoom_factor = if y > 0.0 { 1.1 } else { 0.9 };

                // Zoom X axis
                self.state
                    .get_axis_mut(&AXIS_X)
                    .unwrap()
                    .scale_mut()
                    .zoom(zoom_factor, Some(point.x as f64));

                // Zoom Y axes (using y coordinate as anchor)
                // Note: point.y is normalized 0..1 where 0 is bottom (or top depending on implementation, usually bottom in plot space)
                // Let's check if we need to invert it. Chart widget usually passes normalized coordinates.
                self.state
                    .get_axis_mut(&AXIS_Y_LEFT)
                    .unwrap()
                    .scale_mut()
                    .zoom(zoom_factor, Some(point.y as f64));

                self.state
                    .get_axis_mut(&AXIS_Y_RIGHT)
                    .unwrap()
                    .scale_mut()
                    .zoom(zoom_factor, Some(point.y as f64));
            }
            Message::AxisScrolled(axis_id, normalized_value, delta) => {
                if let Some(axis) = self.state.get_axis_mut(&axis_id) {
                    let y = match delta {
                        ScrollDelta::Lines { y, .. } => y,
                        ScrollDelta::Pixels { y, .. } => y,
                    };
                    let zoom_factor = if y > 0.0 { 1.1 } else { 0.9 };
                    axis.scale_mut()
                        .zoom(zoom_factor, Some(normalized_value as f64));
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // Configure Chart
        let chart = Chart::new(&self.state)
            .layers(&self.layers)
            .on_drag(Message::PanChart)
            .on_axis_drag(Message::AxisDragged)
            .on_axis_hover(Message::AxisHovered)
            .on_scroll(Message::ChartScrolled)
            .on_axis_scroll(Message::AxisScrolled);

        // Controls
        let slider = Slider::new(10..=1000, self.point_count, Message::PointCountChanged);
        let controls = row![
            text("Point Count:"),
            text(self.point_count.to_string()),
            slider
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);

        column![chart, controls].padding(10).spacing(10).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn main() -> iced::Result {
    iced::application(ExampleApp::init, ExampleApp::update, ExampleApp::view)
        .theme(ExampleApp::theme)
        .antialiasing(true)
        .run()
}

fn generate_layers(count: u32) -> Vec<Layer<AxisId, f64>> {
    let mut layers = Vec::new();

    // --- Layer 1: Sine Wave (Left Axis) ---
    let mut layer_left = Layer::new(AXIS_X, AXIS_Y_LEFT);
    let shapes_left: Vec<shape::Shape<f64>> = (0..count)
        .map(|i| {
            let x = i as f64 / 10.0;
            let y = x.sin();
            let pos = PlotPoint::new(x, y);
            let color = Color::from_rgb(0.4, 0.7, 1.0); // Match axis color
            shape::Rectangle::from_center(pos, Length::Screen(3.0), Length::Screen(3.0), color)
                .unwrap()
                .into()
        })
        .collect();
    layer_left.add_shapes(shapes_left);
    layers.push(layer_left);

    // --- Layer 2: Cosine Wave (Right Axis) ---
    // We'll make this one larger amplitude to show off the different scale
    let mut layer_right = Layer::new(AXIS_X, AXIS_Y_RIGHT);
    let shapes_right: Vec<shape::Shape<f64>> = (0..count)
        .map(|i| {
            let x = i as f64 / 10.0;
            let y = x.cos() * 4.0; // Amplitude 4, fits in -5..5 range
            let pos = PlotPoint::new(x, y);
            let color = Color::from_rgb(1.0, 0.6, 0.2); // Match axis color
            shape::Rectangle::from_center(pos, Length::Screen(4.0), Length::Screen(4.0), color)
                .unwrap()
                .into()
        })
        .collect();
    layer_right.add_shapes(shapes_right);
    layers.push(layer_right);

    layers
}
