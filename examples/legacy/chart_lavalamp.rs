//! Lava Lamp Simulator
//!
//! A dynamic, real-time metaball simulation.
//!
//! Demonstrates:
//! 1. Combining a real-time simulation (like the physics sandbox)
//!    with a grid-based renderer (like the fractal explorer).
//! 2. A "Metaball" render algorithm to create fluid, merging shapes.
//! 3. Using Sliders to control simulation and render parameters
//!    in real-time (Speed, Fidelity, Gooeyness).

use iced::{
    Color, Element, Length, Point, Subscription, Task, Theme,
    mouse::ScrollDelta,
    time::Duration,
    widget::{Button, Container, Slider, column, container, row, slider, text},
    window,
};
use iced_extras::widget::chart::{
    self,
    Axis,
    Chart,
    DragDelta,
    Layer,
    PlotPoint,
    PlotRectangle,
    Position,
    State,
    axis::TickLine,
    render::Buffer,
    scale::{Linear, Tick},
    // Use shape::Length for plot coordinates
    shape::{self, Ellipse, Length as PlotLength, Rectangle},
};
use rand::Rng;
use std::{collections::BTreeMap, time::Instant}; // For randomizing blobs

// --- Constants ---

const X_AXIS_ID: AxisId = "x_axis"; // World X coordinate
const Y_AXIS_ID: AxisId = "y_axis"; // World Y coordinate

// Define the simulation world boundaries
const WORLD_BOUNDS: PlotRectangle = PlotRectangle {
    x: 0.0,
    y: 0.0,
    width: 50.0,
    height: 100.0,
};
const BLOB_COUNT: usize = 20; // Was 10

// --- Shared Types ---

/// Type alias for the unique ID of a chart axis.
pub type AxisId = &'static str;

/// Represents a single blob of "wax".
#[derive(Debug)]
struct Blob {
    x: f32,
    y: f32,
    vy: f32, // Only moves vertically
    radius: f32,
    color: Color,
}

/// Defines the messages that can be sent to update the application state.
#[derive(Debug, Clone, Copy)] // Use Copy for simple messages
enum Message {
    /// A message to trigger a full rebuild of the chart layers.
    UpdateChart,
    /// A message sent when the "game loop" ticks.
    Tick(Instant),
    /// A message sent when the main plot area is dragged (for panning).
    OnPlotDrag(DragDelta),
    /// A message sent when an axis is dragged (for zooming).
    OnAxisDrag(AxisId, f32),
    /// A message sent when the scroll wheel is used.
    OnScroll(Point, ScrollDelta),
    /// A message to pause/unpause the simulation.
    TogglePause,
    /// A message to re-seed the blobs.
    Restart,
    /// A message to change the simulation speed.
    SpeedChanged(u32),
    /// A message to change the render fidelity.
    FidelityChanged(u32),
    /// A message to change the "gooeyness" (render threshold).
    ThresholdChanged(f32), // Renamed from GooeynessChanged
}

// --- Main Entry Point ---

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application Structure ---

/// Represents the top-level state of the iced application.
struct ExampleApp {
    lava_lamp: LavaLamp,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let app = Self {
            lava_lamp: LavaLamp::new(),
        };
        (app, Task::done(Message::UpdateChart))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.lava_lamp.handle_message(message);
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        self.lava_lamp.view()
    }

    fn subscription(&self) -> Subscription<Message> {
        self.lava_lamp.subscription()
    }

    const fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .subscription(Self::subscription)
            .theme(Self::theme)
            .antialiasing(true)
            .run()
    }
}

// --- Chart Component ---

/// Manages the state and rendering of the lava lamp.
struct LavaLamp {
    /// The single state managing the viewport into the world.
    state: chart::State<AxisId>,

    /// Layer for drawing the metaballs.
    render_layer: Layer<AxisId>,

    /// The list of all blobs in the simulation.
    blobs: Vec<Blob>,

    /// Whether the simulation is paused.
    paused: bool,

    /// The time of the last simulation tick.
    last_tick: Instant,
    /// The delay between simulation ticks.
    sim_speed_ms: u32,

    /// Render parameters
    fidelity: u32, // The N x N render grid
    // gooeyness: f32, // The influence threshold // REMOVED - Redundant
    threshold: f32, // The influence threshold
}

impl LavaLamp {
    /// Creates a new `LavaLamp`.
    pub fn new() -> Self {
        let initial_x_range = (WORLD_BOUNDS.x, WORLD_BOUNDS.width);
        let initial_y_range = (WORLD_BOUNDS.y, WORLD_BOUNDS.height);

        // --- Create Chart State ---
        let mut state: State<AxisId> = chart::State::new();
        state.set_axis(
            X_AXIS_ID,
            Axis::new(
                Linear::new(initial_x_range.0, initial_x_range.1),
                Position::Bottom,
            ),
        );
        state.set_axis(
            Y_AXIS_ID,
            Axis::new(
                Linear::new(initial_y_range.0, initial_y_range.1),
                Position::Left,
            ),
        );

        let mut lamp = Self {
            state,
            render_layer: Layer::new(X_AXIS_ID, Y_AXIS_ID),
            blobs: Vec::with_capacity(BLOB_COUNT),
            paused: true, // Start paused
            last_tick: Instant::now(),
            sim_speed_ms: 50, // Default 50ms delay
            fidelity: 200,    // 200x200 grid (was 150)
            // gooeyness: 1.0, // Default threshold // REMOVED
            threshold: 1.0, // Default threshold
        };

        lamp.seed_blobs();
        lamp
    }

    /// Creates a new set of random blobs.
    fn seed_blobs(&mut self) {
        self.blobs.clear();
        let mut rng = rand::thread_rng();
        for _ in 0..BLOB_COUNT {
            let radius = rng.gen_range(5.0..=10.0);

            // Create "Hot" and "Cool" blobs
            let (color, vy) = if rng.random_bool(0.5) {
                // Hot Blob (rises)
                (
                    Color::from_rgb(
                        rng.gen_range(0.8..=1.0), // Red
                        rng.gen_range(0.2..=0.5), // Green
                        rng.gen_range(0.1..=0.3), // Blue
                    ),
                    rng.gen_range(1.0..=5.0), // Rises
                )
            } else {
                // Cool Blob (sinks)
                (
                    Color::from_rgb(
                        rng.gen_range(0.1..=0.3), // Red
                        rng.gen_range(0.5..=1.0), // Green
                        rng.gen_range(0.8..=1.0), // Blue
                    ),
                    rng.gen_range(-5.0..=-1.0), // Sinks
                )
            };

            self.blobs.push(Blob {
                x: rng.gen_range(WORLD_BOUNDS.x + radius..=WORLD_BOUNDS.width - radius),
                y: rng.gen_range(WORLD_BOUNDS.y + radius..=WORLD_BOUNDS.height - radius),
                vy, // Use the new vy
                radius,
                color, // Use the new color
            });
        }
    }

    /// Returns the UI elements for the component (controls + chart).
    pub fn view(&self) -> Element<'_, Message> {
        // Create the chart widget
        let chart = Chart::new(&self.state)
            .layer(&self.render_layer)
            .on_drag(Message::OnPlotDrag)
            .on_axis_drag(Message::OnAxisDrag)
            .on_scroll(Message::OnScroll);

        // --- Create the controls ---
        let play_pause_text = if self.paused { "Play" } else { "Pause" };
        let control_row = row![
            Button::new(text(play_pause_text)).on_press(Message::TogglePause),
            Button::new(text("Restart")).on_press(Message::Restart),
        ]
        .spacing(10);

        // Slider for Simulation Speed
        let speed_control = row![
            text(format!("Sim Speed ({}ms):", self.sim_speed_ms)).width(Length::Fixed(150.0)),
            Slider::new(0..=200, self.sim_speed_ms, Message::SpeedChanged)
                .step(10u32)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(10);

        // Slider for Fidelity
        let fidelity_control = row![
            text(format!("Fidelity: {}", self.fidelity)).width(Length::Fixed(150.0)),
            Slider::new(50..=500, self.fidelity, Message::FidelityChanged) // Max 500 (was 300)
                .step(10u32)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(10);

        // Slider for Threshold
        let threshold_control = row![
            text(format!("Threshold: {:.2}", self.threshold)).width(Length::Fixed(150.0)),
            Slider::new(
                0.5..=5.0, // Tweak this range
                self.threshold,
                Message::ThresholdChanged
            )
            .step(0.1)
            .width(Length::Fixed(200.0)),
        ]
        .spacing(10);

        let controls = column![
            control_row,
            speed_control,
            fidelity_control,
            // gooeyness_control // REMOVED
            threshold_control
        ]
        .spacing(5)
        .padding(10);

        // Return a column containing both
        column![controls, chart].into()
    }

    /// Defines the subscription for the "game loop".
    pub fn subscription(&self) -> Subscription<Message> {
        if self.paused {
            Subscription::none()
        } else {
            // Subscribe to rendered frames
            Subscription::batch(vec![window::frames().map(Message::Tick)])
        }
    }

    /// Handles incoming messages and updates the chart state.
    pub fn handle_message(&mut self, message: Message) {
        match message {
            Message::Tick(now) => {
                if self.paused {
                    return;
                }
                if now.duration_since(self.last_tick)
                    >= Duration::from_millis(self.sim_speed_ms as u64)
                {
                    self.update_simulation();
                    self.last_tick = now;
                    self.rebuild_layers();
                }
            }
            Message::UpdateChart => {
                self.rebuild_layers();
            }
            Message::OnPlotDrag(delta) => {
                self.handle_plot_drag(delta);
                self.rebuild_layers();
            }
            Message::OnAxisDrag(id, delta) => {
                self.handle_axis_drag(id, delta);
                self.rebuild_layers();
            }
            Message::OnScroll(point, scroll_delta) => {
                self.handle_scroll_zoom(point, scroll_delta);
                self.rebuild_layers();
            }
            Message::TogglePause => {
                self.paused = !self.paused;
                if !self.paused {
                    self.last_tick = Instant::now();
                }
            }
            Message::Restart => {
                self.seed_blobs();
                self.rebuild_layers();
            }
            Message::SpeedChanged(ms) => {
                self.sim_speed_ms = ms;
            }
            Message::FidelityChanged(f) => {
                self.fidelity = f;
                self.rebuild_layers();
            }
            // REMOVED broken GooeynessChanged handler
            Message::ThresholdChanged(t) => {
                self.threshold = t;
                self.rebuild_layers();
            }
        }
    }

    // --- Event Handlers (Pan/Zoom) ---

    fn handle_plot_drag(&mut self, delta: DragDelta) {
        self.state
            .get_axis_mut(&X_AXIS_ID)
            .unwrap()
            .scale_mut()
            .pan(delta.x);
        self.state
            .get_axis_mut(&Y_AXIS_ID)
            .unwrap()
            .scale_mut()
            .pan(delta.y);
    }

    fn handle_axis_drag(&mut self, id: AxisId, delta: f32) {
        let factor = 1.0 + delta * 2.0;
        let anchor = Some(0.5);
        self.state
            .get_axis_mut(&X_AXIS_ID)
            .unwrap()
            .scale_mut()
            .zoom(factor, anchor);
        self.state
            .get_axis_mut(&Y_AXIS_ID)
            .unwrap()
            .scale_mut()
            .zoom(factor, anchor);
    }

    fn handle_scroll_zoom(&mut self, point: Point, scroll_delta: ScrollDelta) {
        if let ScrollDelta::Lines { x, y } = scroll_delta {
            let factor = 1.1_f32.powf(y);
            self.state
                .get_axis_mut(&X_AXIS_ID)
                .unwrap()
                .scale_mut()
                .zoom(factor, Some(point.x));
            self.state
                .get_axis_mut(&Y_AXIS_ID)
                .unwrap()
                .scale_mut()
                .zoom(factor, Some(point.y));
        }
    }

    // --- Core Logic ---

    /// Clears and rebuilds the render layer.
    fn rebuild_layers(&mut self) {
        self.render_layer.clear();
        let buffer = self.render_layer.buffer_mut();

        // Get the visible domains
        let x_domain = self.state.get_axis(&X_AXIS_ID).unwrap().scale().domain();
        let y_domain = self.state.get_axis(&Y_AXIS_ID).unwrap().scale().domain();

        let (res_x, res_y) = (self.fidelity, self.fidelity);
        let x_step = (x_domain.1 - x_domain.0) / res_x as f32;
        let y_step = (y_domain.1 - y_domain.0) / res_y as f32;

        let rect_width = PlotLength::Plot(x_step);
        let rect_height = PlotLength::Plot(y_step);

        // Pre-calculate blob colors (or just use one)
        let blob_color = Color::from_rgb(1.0, 0.3, 0.1); // Orange-Red
        // Define our background gradient
        let liquid_color_hot = Color::from_rgb(0.3, 0.0, 0.1); // Dark Red (bottom)
        let liquid_color_cool = Color::from_rgb(0.1, 0.0, 0.3); // Dark Purple (top)

        for i in 0..res_x {
            for j in 0..res_y {
                // Center of this render "pixel"
                let px = x_domain.0 + (i as f32 + 0.5) * x_step;
                let py = y_domain.0 + (j as f32 + 0.5) * y_step;

                // --- 1. Calculate Background Color ---
                // Get vertical position as 0.0-1.0
                let y_percent = (py - y_domain.0) / (y_domain.1 - y_domain.0);
                let background_color = Self::lerp_color(
                    liquid_color_hot,
                    liquid_color_cool,
                    y_percent.clamp(0.0, 1.0),
                );

                let mut total_influence = 0.0;
                // We'll also calculate a weighted average color
                let mut total_r = 0.0;
                let mut total_g = 0.0;
                let mut total_b = 0.0;

                for blob in &self.blobs {
                    // Use distance squared for performance
                    let dx = px - blob.x;
                    let dy = py - blob.y;
                    let dist_sq = dx * dx + dy * dy;

                    // Use r^2 as the influence for a sharper falloff
                    let influence = (blob.radius * blob.radius) / dist_sq;
                    total_influence += influence;

                    // Add this blob's color, weighted by its influence
                    total_r += influence * blob.color.r;
                    total_g += influence * blob.color.g;
                    total_b += influence * blob.color.b;
                }

                // --- New Coloring Logic ---
                let color = if total_influence > 0.001 {
                    // Avoid divide-by-zero
                    // 1. Calculate the mixed color of the wax
                    let mixed_r = (total_r / total_influence).min(1.0);
                    let mixed_g = (total_g / total_influence).min(1.0);
                    let mixed_b = (total_b / total_influence).min(1.0);
                    let mixed_color = Color::from_rgb(mixed_r, mixed_g, mixed_b);

                    // 2. Calculate the "heat" (0.0 to 1.0)
                    let heat = (total_influence / self.threshold).min(1.0);

                    // 3. Lerp from the dynamic background to the hot mixed color
                    Self::lerp_color(background_color, mixed_color, heat)
                } else {
                    background_color // We're in empty space
                };
                // --- End New Coloring Logic ---

                let rect = Rectangle {
                    position: PlotPoint::new(px, py),
                    h_anchor: shape::HorizontalOrientation::Center,
                    v_anchor: shape::VerticalOrientation::Center, // FIXED TYPO
                    width: rect_width,
                    height: rect_height,
                    fill: Some(color),
                    stroke: None,
                };
                buffer.push(rect);
            }
        }
    }

    /// Runs one step of the physics simulation.
    fn update_simulation(&mut self) {
        let bounds = (WORLD_BOUNDS.y, WORLD_BOUNDS.height); // min y, max y
        let mut rng = rand::thread_rng();

        for blob in &mut self.blobs {
            // Update position
            blob.y += blob.vy * 0.1; // 0.1 is a fixed delta-t fudge factor

            // Bounce off top and bottom
            if blob.y - blob.radius < bounds.0 {
                blob.y = bounds.0 + blob.radius;
                blob.vy = rng.gen_range(5.0..=10.0); // Heat up: go up (was 1-5)
            }
            if blob.y + blob.radius > bounds.1 {
                blob.y = bounds.1 - blob.radius;
                blob.vy = rng.gen_range(-10.0..=-5.0); // Cool down: go down (was -5 to -1)
            }

            // Simple horizontal drift
            blob.x += rng.gen_range(-0.1..=0.1);
            // Keep in horizontal bounds
            blob.x = blob.x.clamp(
                WORLD_BOUNDS.x + blob.radius,
                WORLD_BOUNDS.width - blob.radius,
            );
        }
    }

    /// Linearly interpolates between two colors.
    fn lerp_color(a: Color, b: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: a.r + (b.r - a.r) * t,
            g: a.g + (b.g - a.g) * t,
            b: a.b + (b.b - a.b) * t,
            a: a.a + (b.a - a.a) * t,
        }
    }
}
