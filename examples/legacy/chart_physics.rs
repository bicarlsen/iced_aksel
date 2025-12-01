//! 2D Physics Sandbox
//!
//! A dynamic, real-time physics simulation.
//!
//! Demonstrates:
//! 1. Using `iced::subscription::window::frames` for a "game loop".
//! 2. Simulating dynamic, non-grid data (`Vec<Ball>`).
//! 3. Using Pan/Zoom/Scroll to explore the 2D world.
//! 4. Using multiple layers: one for objects, one for visual "tracers".
//! 5. Integrating Buttons/Sliders to interact with the simulation
//!    (add balls, change gravity).

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
use std::{
    collections::{BTreeMap, VecDeque}, // Use VecDeque for tracers
    time::Instant,
}; // For randomizing balls

// --- Constants ---

const X_AXIS_ID: AxisId = "x_axis"; // World X coordinate
const Y_AXIS_ID: AxisId = "y_axis"; // World Y coordinate

// Define the simulation world boundaries
const WORLD_BOUNDS: PlotRectangle = PlotRectangle {
    x: 0.0,
    y: 0.0,
    width: 100.0,
    height: 100.0,
};
const TRACE_LENGTH: usize = 20;

// --- Shared Types ---

/// Type alias for the unique ID of a chart axis.
pub type AxisId = &'static str;

/// Represents a single ball in the physics simulation.
#[derive(Debug)]
struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
    color: Color,
    /// A queue of the last N positions for the tracer
    trace: VecDeque<PlotPoint>,
}

/// Defines the messages that can be sent to update the application state.
#[derive(Debug, Clone)]
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
    /// A message to add a new random ball.
    AddBall,
    /// A message to change the gravity.
    GravityChanged(f32),
    /// A message to change the bounciness.
    BouncinessChanged(f32),
}

// --- Main Entry Point ---

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application Structure ---

/// Represents the top-level state of the iced application.
struct ExampleApp {
    sandbox: PhysicsSandbox,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let app = Self {
            sandbox: PhysicsSandbox::new(),
        };
        (app, Task::done(Message::UpdateChart))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.sandbox.handle_message(message);
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        self.sandbox.view()
    }

    fn subscription(&self) -> Subscription<Message> {
        self.sandbox.subscription()
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

/// Manages the state and rendering of the physics sandbox.
struct PhysicsSandbox {
    /// The single state managing the viewport into the world.
    state: chart::State<AxisId>,

    /// Layer for drawing the balls.
    balls_layer: Layer<AxisId>,
    /// Layer for drawing the tracers.
    tracers_layer: Layer<AxisId>,

    /// The list of all balls in the simulation.
    balls: Vec<Ball>,

    /// Whether the simulation is paused.
    paused: bool,

    /// The time of the last simulation tick.
    last_tick: Instant,

    /// Simulation parameters
    gravity: f32,
    bounciness: f32,
}

impl PhysicsSandbox {
    /// Creates a new `PhysicsSandbox`.
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

        let mut sandbox = Self {
            state,
            balls_layer: Layer::new(X_AXIS_ID, Y_AXIS_ID),
            tracers_layer: Layer::new(X_AXIS_ID, Y_AXIS_ID),
            balls: Vec::new(),
            paused: true, // Start paused
            last_tick: Instant::now(),
            gravity: 20.0,   // A good starting gravity
            bounciness: 0.8, // Good bounciness
        };

        // Start with a few balls
        sandbox.add_ball();
        sandbox.add_ball();

        sandbox
    }

    /// Adds a new random ball to the simulation.
    fn add_ball(&mut self) {
        let mut rng = rand::thread_rng();
        let radius = rng.gen_range(1.0..=3.0);
        let ball = Ball {
            x: rng.gen_range(WORLD_BOUNDS.x + radius..=WORLD_BOUNDS.width - radius),
            y: rng.gen_range(WORLD_BOUNDS.y + radius..=WORLD_BOUNDS.height - radius),
            vx: rng.gen_range(-20.0..=20.0),
            vy: rng.gen_range(-20.0..=20.0),
            radius,
            color: Color::from_rgb(
                rng.gen_range(0.5..=1.0),
                rng.gen_range(0.5..=1.0),
                rng.gen_range(0.5..=1.0),
            ),
            trace: VecDeque::with_capacity(TRACE_LENGTH),
        };
        self.balls.push(ball);
    }

    /// Returns the UI elements for the component (controls + chart).
    pub fn view(&self) -> Element<'_, Message> {
        // Create the chart widget
        let chart = Chart::new(&self.state)
            // Draw tracers first (bottom layer)
            .layer(&self.tracers_layer)
            // Draw balls on top
            .layer(&self.balls_layer)
            .on_drag(Message::OnPlotDrag)
            .on_axis_drag(Message::OnAxisDrag)
            .on_scroll(Message::OnScroll);

        // --- Create the controls ---
        let play_pause_text = if self.paused { "Play" } else { "Pause" };
        let control_row = row![
            Button::new(text(play_pause_text)).on_press(Message::TogglePause),
            Button::new(text("Add Ball")).on_press(Message::AddBall),
        ]
        .spacing(10);

        // Slider for Gravity
        let gravity_control = row![
            text(format!("Gravity: {:.1}", self.gravity)).width(Length::Fixed(150.0)),
            Slider::new(0.0..=50.0, self.gravity, Message::GravityChanged)
                .step(0.5)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(10);

        // Slider for Bounciness
        let bounciness_control = row![
            text(format!("Bounciness: {:.2}", self.bounciness)).width(Length::Fixed(150.0)),
            Slider::new(
                0.1..=1.0, // 1.0 = perfect bounce
                self.bounciness,
                Message::BouncinessChanged
            )
            .step(0.05)
            .width(Length::Fixed(200.0)),
        ]
        .spacing(10);

        let controls = column![control_row, gravity_control, bounciness_control]
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
            // This is the game loop!
            Message::Tick(now) => {
                // Don't run sim if paused
                if self.paused {
                    return;
                }

                // Calculate delta time (dt) for physics
                let dt = now.duration_since(self.last_tick).as_secs_f32();
                self.last_tick = now; // Reset the tick timer

                // Cap dt to prevent "spiral of death" if lagging
                let dt = dt.min(0.05);

                self.update_simulation(dt);
                // We MUST rebuild the layer to show the new state.
                self.rebuild_layers();
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
                // If we just un-paused, reset the tick timer
                if !self.paused {
                    self.last_tick = Instant::now();
                }
            }
            Message::AddBall => {
                self.add_ball();
                self.rebuild_layers(); // Rebuild to show the new ball
            }
            Message::GravityChanged(g) => {
                self.gravity = g;
            }
            Message::BouncinessChanged(b) => {
                self.bounciness = b;
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

    /// Clears and rebuilds all shape layers.
    fn rebuild_layers(&mut self) {
        self.balls_layer.clear();
        self.tracers_layer.clear();

        for ball in &self.balls {
            // --- 1. Draw the Ball ---
            let ball_shape = Ellipse::new(
                PlotPoint::new(ball.x, ball.y),
                PlotLength::Plot(ball.radius * 2.0), // width
                PlotLength::Plot(ball.radius * 2.0), // height
                ball.color,
            );
            self.balls_layer.buffer_mut().push(ball_shape);

            // --- 2. Draw the Tracer ---
            for (i, p) in ball.trace.iter().enumerate() {
                // Calculate alpha: fades from 0.5 to 0.0
                let alpha = 0.5 * (1.0 - (i as f32 / TRACE_LENGTH as f32));
                let color = Color {
                    a: alpha,
                    ..ball.color
                };

                // Calculate size: tracer dots get smaller
                let size_perc = 1.0 - (i as f32 / TRACE_LENGTH as f32);
                let size = PlotLength::Plot(ball.radius * size_perc);

                let trace_dot = Ellipse::new(*p, size, size, color);
                self.tracers_layer.buffer_mut().push(trace_dot);
            }
        }
    }

    /// Runs one step of the physics simulation.
    fn update_simulation(&mut self, dt: f32) {
        let bounds = (
            WORLD_BOUNDS.x,
            WORLD_BOUNDS.width, // max x
            WORLD_BOUNDS.y,
            WORLD_BOUNDS.height, // max y
        );

        // --- 1. Apply forces and update positions ---
        for ball in &mut self.balls {
            // Apply gravity
            ball.vy -= self.gravity * dt;
            // Update position
            ball.x += ball.vx * dt;
            ball.y += ball.vy * dt;
        }

        // --- 2. Handle Ball-to-Ball Collisions ---
        // We must use a more complex loop structure to satisfy the borrow checker
        // when mutating two balls at once.
        let mut i = 0;
        while i < self.balls.len() {
            // split_at_mut gives us a mutable slice of balls before `i+1`
            // and a mutable slice of balls from `i+1` onwards.
            let (left, right) = self.balls.split_at_mut(i + 1);
            let ball_a = &mut left[i]; // The ball we are checking

            for ball_b in right.iter_mut() {
                // Now we can safely mutate ball_a and ball_b
                let dx = ball_b.x - ball_a.x;
                let dy = ball_b.y - ball_a.y;
                let distance_sq = dx * dx + dy * dy;
                let radii_sum = ball_a.radius + ball_b.radius;

                // Check for collision
                if distance_sq < radii_sum * radii_sum {
                    // Pass bounciness as an argument
                    Self::resolve_ball_collision(ball_a, ball_b, self.bounciness);
                }
            }
            i += 1;
        }

        // --- 3. Handle Wall Collisions & Update Tracers ---
        for ball in &mut self.balls {
            // Update Tracer
            ball.trace.push_front(PlotPoint::new(ball.x, ball.y));
            if ball.trace.len() > TRACE_LENGTH {
                ball.trace.pop_back();
            }

            // Handle Wall Collisions
            // Left wall
            if ball.x - ball.radius < bounds.0 {
                ball.x = bounds.0 + ball.radius;
                ball.vx *= -self.bounciness;
            }
            // Right wall
            if ball.x + ball.radius > bounds.1 {
                ball.x = bounds.1 - ball.radius;
                ball.vx *= -self.bounciness;
            }
            // Bottom wall
            if ball.y - ball.radius < bounds.2 {
                ball.y = bounds.2 + ball.radius;
                ball.vy *= -self.bounciness;
            }
            // Top wall
            if ball.y + ball.radius > bounds.3 {
                ball.y = bounds.3 - ball.radius;
                ball.vy *= -self.bounciness;
            }
        }
    }

    /// Resolves collision between two mutable balls
    fn resolve_ball_collision(a: &mut Ball, b: &mut Ball, bounciness: f32) {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let distance = (dx * dx + dy * dy).sqrt();
        let radii_sum = a.radius + b.radius;

        // --- 1. Static Resolution (Move them apart) ---
        // We move them apart based on how much they overlap
        let overlap = radii_sum - distance;
        let move_x = (overlap / 2.0) * (dx / distance);
        let move_y = (overlap / 2.0) * (dy / distance);

        a.x -= move_x;
        a.y -= move_y;
        b.x += move_x;
        b.y += move_y;

        // --- 2. Dynamic Resolution (Bounce) ---
        // Normalized collision axis (vector from a to b)
        let nx = dx / distance;
        let ny = dy / distance;

        // Relative velocity
        let rvx = b.vx - a.vx;
        let rvy = b.vy - a.vy;

        // Velocity along the normal (dot product)
        let vel_along_normal = rvx * nx + rvy * ny;

        // Don't resolve if they're already moving apart
        if vel_along_normal > 0.0 {
            return;
        }

        // Calculate impulse (assuming equal mass)
        // We apply bounciness here
        let impulse = -2.0 * vel_along_normal * bounciness;

        // Apply impulse (split between two balls)
        a.vx -= (impulse * nx) / 2.0;
        a.vy -= (impulse * ny) / 2.0;
        b.vx += (impulse * nx) / 2.0;
        b.vy += (impulse * ny) / 2.0;
    }
}
