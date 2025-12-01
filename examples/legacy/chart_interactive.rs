//! Chart shape layer example

use iced::{
    Color, Element, Event, Point, Task, Theme,
    keyboard::{self, Modifiers, on_key_press},
    mouse::ScrollDelta,
    widget::{Slider, column, text},
};
use iced_extras::widget::chart::{
    Axis, Chart, DragDelta, Layer, Orientation, PlotPoint, PlotRectangle, Position, State,
    scale::Linear,
    shape::{self, Length},
};

const LINEAR_AXIS_ID_X: &str = "linear_x";
const LINEAR_AXIS_ID_Y: &str = "linear_y";

/// Define the plot's boundaries once.
const PLOT_BOUNDS: PlotRectangle = PlotRectangle {
    x: 0.,
    y: 0.,
    width: 100.,
    height: 100.,
};

type AxisId = &'static str;

#[derive(Debug, Clone)]
enum Message {
    UpdateShapes(usize),
    OnModifiersChanged(Modifiers),
    OnClick(Point),
    OnDrag(DragDelta),
    OnScroll(Point, ScrollDelta),
    OnClickAxis(AxisId, f32),
    OnDragAxis(AxisId, f32),
    OnScrollAxis(AxisId, f32, ScrollDelta),
}

struct ExampleApp {
    state: State<AxisId>,

    // Layer buffer. This is all `Shape` and the axis it belongs to
    plot_shapes: Layer<AxisId>,

    modifiers: Modifiers,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut state = State::new();

        // Use the constant for axis setup
        let axis_x = Axis::new(
            Linear::new(PLOT_BOUNDS.x, PLOT_BOUNDS.x + PLOT_BOUNDS.width),
            Position::Bottom,
        );

        let axis_y = Axis::new(
            Linear::new(PLOT_BOUNDS.y, PLOT_BOUNDS.y + PLOT_BOUNDS.height),
            Position::Left,
        );

        state.set_axis(LINEAR_AXIS_ID_X, axis_x);
        state.set_axis(LINEAR_AXIS_ID_Y, axis_y);

        let app = Self {
            state,
            plot_shapes: Layer::new(LINEAR_AXIS_ID_X, LINEAR_AXIS_ID_Y),
            modifiers: Modifiers::empty(),
        };

        // Start with 500 shapes so the app isn't empty
        (app, Task::done(Message::UpdateShapes(500)))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateShapes(count) => {
                // Pass the constant to the generator
                let new_shapes = generate_bouncing_pattern(count, PLOT_BOUNDS);
                self.plot_shapes.replace(new_shapes);
            }
            Message::OnModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }
            Message::OnClick(point) => {
                println!("{point:?}");
            }
            Message::OnDrag(delta) => {
                self.state
                    .get_axis_mut(&LINEAR_AXIS_ID_X)
                    .unwrap()
                    .scale_mut()
                    .pan(delta.x);
                self.state
                    .get_axis_mut(&LINEAR_AXIS_ID_Y)
                    .unwrap()
                    .scale_mut()
                    .pan(delta.y);
            }
            Message::OnScroll(cursor_pos, delta) => {
                println!("CURSOR: {cursor_pos:?}, DELTA: {delta:#?}");
                match delta {
                    ScrollDelta::Lines { x, y } => {
                        // Each scroll line = 10% zoom change
                        // y = 1.0 (scroll up) → factor = 1.1 (zoom in)
                        // y = -1.0 (scroll down) → factor = ~0.909 (zoom out)
                        let factor = 1.1_f32.powf(y);

                        self.state
                            .get_axis_mut(&LINEAR_AXIS_ID_X)
                            .unwrap()
                            .scale_mut()
                            .zoom(factor, Some(cursor_pos.x));
                        self.state
                            .get_axis_mut(&LINEAR_AXIS_ID_Y)
                            .unwrap()
                            .scale_mut()
                            .zoom(factor, Some(cursor_pos.y));
                    }
                    ScrollDelta::Pixels { x, y } => {
                        // For pixel-based scrolling (touchpad)
                        // Divide by larger number for less sensitive zooming
                        let factor = 1.0 + y / 500.0;

                        self.state
                            .get_axis_mut(&LINEAR_AXIS_ID_X)
                            .unwrap()
                            .scale_mut()
                            .zoom(factor, Some(cursor_pos.x));
                        self.state
                            .get_axis_mut(&LINEAR_AXIS_ID_Y)
                            .unwrap()
                            .scale_mut()
                            .zoom(factor, Some(cursor_pos.y));
                    }
                }
            }
            Message::OnClickAxis(id, point) => {
                let point = self.state.get_axis(&id).unwrap().scale().denormalize(point);
                println!("Clicked axis: {id} at {point}");
            }
            Message::OnDragAxis(id, delta) => {
                let axis = self.state.get_axis_mut(&id).unwrap();
                let orientation = axis.orientation();
                let scale = axis.scale_mut();

                // Convert normalized delta to zoom factor
                // Multiply by larger number for more sensitivity
                let factor = 1.0 + delta * 2.0;

                match orientation {
                    Orientation::Horizontal => scale.zoom(factor, Some(1.0)),
                    Orientation::Vertical => scale.zoom(factor, Some(0.5)),
                }
            }
            Message::OnScrollAxis(id, point, delta) => {
                let axis = self.state.get_axis_mut(&id).unwrap();
                let orientation = axis.orientation();
                let scale = axis.scale_mut();
                let factor = match delta {
                    ScrollDelta::Lines { y, .. } => 1.1_f32.powf(y),
                    ScrollDelta::Pixels { y, .. } => 1.0 + y / 500.0,
                };

                // If CONTROL is held, anchor from the point the cursor is at
                if self.modifiers.control() {
                    scale.zoom(factor, Some(point));
                    return Task::none();
                }

                match orientation {
                    Orientation::Horizontal => scale.zoom(factor, Some(1.0)),
                    Orientation::Vertical => scale.zoom(factor, Some(0.5)),
                }
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .layer(&self.plot_shapes)
            .on_click(Message::OnClick)
            .on_drag(Message::OnDrag)
            .on_scroll(Message::OnScroll)
            .on_axis_click(Message::OnClickAxis)
            .on_axis_drag(Message::OnDragAxis)
            .on_axis_scroll(Message::OnScrollAxis);

        let slider = Slider::new(0..=2500, self.plot_shapes.len() as i32, |value| {
            Message::UpdateShapes(value as usize)
        })
        .step(1);

        column![chart, slider].into()
    }

    const fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .antialiasing(true)
            .subscription(|_| {
                iced::event::listen_with(|event, _, _| match event {
                    Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                        Some(Message::OnModifiersChanged(modifiers))
                    }
                    _ => None,
                })
            })
            .run()
    }
}

fn main() -> iced::Result {
    ExampleApp::run()
}

/// Generates a `Vec<Shape>` that follows a bouncing path with a pulsing rainbow color.
fn generate_bouncing_pattern(count: usize, bounds: PlotRectangle) -> Vec<shape::Shape> {
    // --- Define the simulation boundaries ---
    let min_x = bounds.x;
    let max_x = bounds.x + bounds.width;
    let min_y = bounds.y;
    let max_y = bounds.y + bounds.height;

    // --- Define the particle's state ---
    let mut pos = PlotPoint::new(
        bounds.x + bounds.width / 2.0,
        bounds.y + bounds.height / 2.0,
    );
    let mut vel = (1.3, 0.9);
    let speed = (bounds.width.min(bounds.height)) / 50.0;
    let shape_size = Length::Plot(2.0);

    // --- New Color State ---
    let mut color_step = 0.0_f32;
    let color_speed = 0.03_f32; // How fast the color changes per step

    // Phase offsets for R, G, B to get a nice rainbow cycle
    // (0 degrees, 120 degrees, 240 degrees)
    let r_phase = 0.0;
    let g_phase = 2.0 * std::f32::consts::PI / 3.0;
    let b_phase = 4.0 * std::f32::consts::PI / 3.0;

    // --- Generate the iterator ---
    std::iter::from_fn(move || {
        // Get the current position *before* updating it
        let current_pos = pos;

        // --- 1. Update the particle's position ---
        pos.x += vel.0 * speed;
        pos.y += vel.1 * speed;

        // --- 2. Check for bounces and update state ---
        if pos.x <= min_x || pos.x >= max_x {
            vel.0 *= -1.0;
            pos.x = pos.x.clamp(min_x, max_x);
        }
        // *** LOGIC BUG FIX HERE: Was max_x, now is max_y ***
        if pos.y <= min_y || pos.y >= max_y {
            vel.1 *= -1.0;
            pos.y = pos.y.clamp(min_y, max_y);
        }

        // --- 3. Update color based on the sine wave "time" ---
        color_step += color_speed;

        // Calculate each channel using a sine wave
        // (sin(x) * 0.5 + 0.5) maps the range [-1, 1] to [0, 1]
        let r_val = (color_step + r_phase).sin() * 0.5 + 0.5;
        let g_val = (color_step + g_phase).sin() * 0.5 + 0.5;
        let b_val = (color_step + b_phase).sin() * 0.5 + 0.5;

        // Convert [0, 1] floats to [0, 255] u8
        let current_color = Color::from_rgb8(
            (r_val * 255.0) as u8,
            (g_val * 255.0) as u8,
            (b_val * 255.0) as u8,
        );

        // --- 4. Return the new shape ---
        Some(
            shape::Rectangle::from_center(current_pos, shape_size, shape_size, current_color)
                .into(),
        )
    })
    .take(count)
    .collect()
}
