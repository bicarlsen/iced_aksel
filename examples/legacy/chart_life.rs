//! Conway's Game of Life Explorer
//!
//! A dynamic, real-time simulation example.
//!
//! Demonstrates:
//! 1. Using `iced::subscription::window::frames` to create a game loop.
//! 2. Updating a simulation state (`world`) on each tick.
//! 3. Using Pan (`OnPlotDrag`) and Zoom (`OnScroll`) to explore
//!    the large, dynamic simulation grid.
//! 4. Integrating `iced::widget::Button`s to control the
//!    simulation (Pause/Play, Restart).
//! 5. Using `shape::Rectangle` to render the state of the world.
//! 6. **NEW:** Generational coloring (cells "age" and change color).
//! 7. **NEW:** Simulation speed control.

use iced::{
    Color,
    Element,
    Length,
    Point,
    Subscription,
    Task,
    Theme,
    mouse::ScrollDelta,
    time::Duration, // Need this for sim speed
    widget::{Button, Container, Slider, column, container, row, slider, text},
    window, // Use the window subscription
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
use std::{collections::BTreeMap, time::Instant}; // Need Instant for the Tick

// --- Constants ---

const X_AXIS_ID: AxisId = "x_axis"; // World X coordinate
const Y_AXIS_ID: AxisId = "y_axis"; // World Y coordinate
const WORLD_SIZE: usize = 256; // The world is a 256x256 grid
const MAX_CELL_AGE: u32 = 60; // Cell "dies" of old age visually after 60 frames

// --- Shared Types ---

/// Type alias for the unique ID of a chart axis.
pub type AxisId = &'static str;

/// Represents the state of our 2D world grid.
/// 0 = Dead, > 0 = Alive (cell's age)
type World = Vec<Vec<u32>>;

/// Defines the messages that can be sent to update the application state.
#[derive(Debug, Clone)]
enum Message {
    /// A message to trigger a full rebuild of the chart layers.
    UpdateChart,
    /// A message sent when the "game loop" ticks.
    Tick(Instant), // Correct signature
    /// A message sent when the main plot area is dragged (for panning).
    OnPlotDrag(DragDelta),
    /// A message sent when an axis is dragged (for zooming).
    OnAxisDrag(AxisId, f32),
    /// A message sent when the scroll wheel is used.
    OnScroll(Point, ScrollDelta),
    /// A message to pause/unpause the simulation.
    TogglePause,
    /// A message to re-seed the world with new random data.
    Restart,
    /// A message to change the simulation speed.
    SpeedChanged(u32),
}

// --- Main Entry Point ---

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application Structure ---

/// Represents the top-level state of the iced application.
struct ExampleApp {
    explorer: LifeExplorer,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let app = Self {
            explorer: LifeExplorer::new(),
        };
        (app, Task::done(Message::UpdateChart))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        self.explorer.handle_message(message);
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        self.explorer.view()
    }

    /// This is the new, crucial part.
    /// We tell `iced` to run our subscription.
    fn subscription(&self) -> Subscription<Message> {
        self.explorer.subscription()
    }

    const fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .subscription(Self::subscription) // <-- Tell iced to use the subscription
            .theme(Self::theme)
            .antialiasing(true)
            .run()
    }
}

// --- Chart Component ---

/// Manages the state and rendering of the Game of Life explorer.
struct LifeExplorer {
    /// The single state managing the viewport into the world.
    /// The scale will be Linear<f32>.
    state: chart::State<AxisId>,

    /// Layer for drawing the cells.
    render_layer: Layer<AxisId>,

    /// The grid of "alive" (true) or "dead" (false) cells.
    world: World,
    /// A buffer to calculate the *next* state of the world.
    world_buffer: World,

    /// Whether the simulation is paused.
    paused: bool,

    /// The time of the last simulation tick.
    last_tick: Instant,
    /// The delay between simulation ticks.
    sim_speed_ms: u32,
}

impl LifeExplorer {
    /// Creates a new `LifeExplorer`.
    pub fn new() -> Self {
        // --- Define Initial Ranges (f32) ---
        // The world coordinates are (0.0 .. 256.0)
        let initial_x_range = (0.0, WORLD_SIZE as f32);
        let initial_y_range = (0.0, WORLD_SIZE as f32);

        // --- Create Chart State ---
        let mut state: State<AxisId> = chart::State::new();
        state.set_axis(
            X_AXIS_ID,
            Axis::new(
                // Use f32
                Linear::new(initial_x_range.0, initial_x_range.1),
                Position::Bottom,
            ),
        );
        state.set_axis(
            Y_AXIS_ID,
            Axis::new(
                // Use f32
                Linear::new(initial_y_range.0, initial_y_range.1),
                Position::Left,
            ),
        );

        // Create an initial random world
        let mut explorer = Self {
            state,
            render_layer: Layer::new(X_AXIS_ID, Y_AXIS_ID),
            world: vec![vec![0; WORLD_SIZE]; WORLD_SIZE], // Now u32
            world_buffer: vec![vec![0; WORLD_SIZE]; WORLD_SIZE], // Now u32
            paused: true,                                 // Start paused
            last_tick: Instant::now(),
            sim_speed_ms: 50, // Default 50ms delay
        };
        explorer.seed_world(); // Populate the world
        explorer
    }

    /// Generates a new random world state.
    fn seed_world(&mut self) {
        for x in 0..WORLD_SIZE {
            for y in 0..WORLD_SIZE {
                // Set to 1 (alive, age 1) or 0 (dead)
                self.world[x][y] = if rand::random::<f32>() > 0.75 { 1 } else { 0 }; // 25% alive
            }
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

        // Slider for simulation speed
        let speed_control = row![
            text(format!("Sim Speed ({}ms):", self.sim_speed_ms)).width(Length::Fixed(150.0)),
            Slider::new(
                0..=200, // 0ms (max) to 200ms (slow)
                self.sim_speed_ms,
                Message::SpeedChanged
            )
            .step(10u32)
            .width(Length::Fixed(200.0)),
        ]
        .spacing(10);

        let controls = column![control_row, speed_control].spacing(5).padding(10);

        // Return a column containing both
        column![controls, chart].into()
    }

    /// Defines the subscription for the "game loop".
    pub fn subscription(&self) -> Subscription<Message> {
        if self.paused {
            // If paused, we send no ticks.
            Subscription::none()
        } else {
            // Use the window::frames subscription
            Subscription::batch(vec![window::frames().map(Message::Tick)])
        }
    }

    /// Handles incoming messages and updates the chart state.
    pub fn handle_message(&mut self, message: Message) {
        match message {
            // This is the game loop!
            Message::Tick(now) => {
                // Check if enough time has passed to update the simulation
                if now.duration_since(self.last_tick)
                    >= Duration::from_millis(self.sim_speed_ms as u64)
                {
                    self.update_simulation();
                    self.last_tick = now; // Reset the tick timer
                    // We MUST rebuild the layer to show the new state.
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
                // If we just un-paused, reset the tick timer
                if !self.paused {
                    self.last_tick = Instant::now();
                }
            }
            Message::Restart => {
                self.seed_world();
                self.rebuild_layers();
            }
            Message::SpeedChanged(ms) => {
                self.sim_speed_ms = ms;
            }
        }
    }

    // --- Event Handlers ---

    fn handle_plot_drag(&mut self, delta: DragDelta) {
        // All f32
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
        // All f32
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
            // All f32
            let factor = 1.1_f32.powf(y);
            self.state
                .get_axis_mut(&X_AXIS_ID)
                .unwrap()
                .scale_mut()
                .zoom(factor, Some(point.x)); // point.x is f32
            self.state
                .get_axis_mut(&Y_AXIS_ID)
                .unwrap()
                .scale_mut()
                .zoom(factor, Some(point.y)); // point.y is f32
        }
    }

    // --- Core Logic ---

    /// Clears and rebuilds the render layer based on the current world state.
    fn rebuild_layers(&mut self) {
        self.render_layer.clear();

        // domain is (f32, f32)
        let x_domain = self.state.get_axis(&X_AXIS_ID).unwrap().scale().domain();
        let y_domain = self.state.get_axis(&Y_AXIS_ID).unwrap().scale().domain();

        // Define the size of a single cell in Plot coordinates.
        let rect_width = PlotLength::Plot(1.0);
        let rect_height = PlotLength::Plot(1.0);

        // Find the range of cells to check.
        let x_min = (x_domain.0.floor() as isize - 1).max(0) as usize;
        let x_max = (x_domain.1.ceil() as isize + 1).min(WORLD_SIZE as isize - 1) as usize;
        let y_min = (y_domain.0.floor() as isize - 1).max(0) as usize;
        let y_max = (y_domain.1.ceil() as isize + 1).min(WORLD_SIZE as isize - 1) as usize;

        for x in x_min..=x_max {
            for y in y_min..=y_max {
                let age = self.world[x][y];
                // Only draw alive cells
                if age > 0 {
                    // Calculate the center of this cell
                    let cx = x as f32 + 0.5;
                    let cy = y as f32 + 0.5;

                    // Get color based on age
                    let color = self.age_to_color(age);

                    let rect = Rectangle {
                        position: PlotPoint::new(cx, cy), // (f32, f32)
                        h_anchor: shape::HorizontalOrientation::Center,
                        v_anchor: shape::VerticalOrientation::Center,
                        width: rect_width,
                        height: rect_height,
                        fill: Some(color),
                        stroke: None,
                    };
                    self.render_layer.buffer_mut().push(rect);
                }
            }
        }
    }

    /// Maps a cell's age to a color.
    fn age_to_color(&self, age: u32) -> Color {
        // Clamp age to max age for color calculation
        let age_f = (age.min(MAX_CELL_AGE)) as f32 / MAX_CELL_AGE as f32;

        // Simple "hot" to "cool" gradient
        // Starts at (1.0, 1.0, 0.0) -> Yellow
        // Fades to (1.0, 0.0, 0.0) -> Red
        // Fades to (0.5, 0.0, 0.5) -> Purple

        let r = 1.0;
        let g = 1.0 - (age_f * 2.0).min(1.0); // Fades from Yellow to Red
        let b = (age_f - 0.5).max(0.0) * 1.0; // Fades from Red to Purple (ish)

        // Special case for brand new cells
        if age == 1 {
            Color::from_rgb(1.0, 1.0, 1.0) // New cells are pure white for a "flash"
        } else {
            Color::from_rgb(r, g, b)
        }
    }

    /// Checks if a cell is alive, with bounds checking.
    fn is_cell_alive(&self, x: usize, y: usize) -> bool {
        if x >= WORLD_SIZE || y >= WORLD_SIZE {
            false
        } else {
            self.world[x][y] > 0 // Now check age
        }
    }

    /// Runs one step of the Game of Life simulation.
    fn update_simulation(&mut self) {
        for x in 0..WORLD_SIZE {
            for y in 0..WORLD_SIZE {
                let alive_neighbors = self.count_alive_neighbors(x, y);
                let cell_age = self.world[x][y];
                let cell_is_alive = cell_age > 0;

                let next_state = match (cell_is_alive, alive_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours dies.
                    (true, n) if n < 2 => 0, // Die
                    // Rule 2: Any live cell with two or three live neighbours lives.
                    (true, 2) | (true, 3) => (cell_age + 1).min(MAX_CELL_AGE), // Get older
                    // Rule 3: Any live cell with more than three live neighbours dies.
                    (true, n) if n > 3 => 0, // Die
                    // Rule 4: Any dead cell with exactly three live neighbours becomes a live cell.
                    (false, 3) => 1, // Born!
                    // All other cases: cell stays in its current state.
                    (true, _) => (cell_age + 1).min(MAX_CELL_AGE), // Should be covered, but good to be safe
                    (false, _) => 0,                               // Stay dead
                };
                self.world_buffer[x][y] = next_state;
            }
        }
        // Swap the buffers so the new state becomes the current state
        std::mem::swap(&mut self.world, &mut self.world_buffer);
    }

    /// Counts alive neighbors for a cell, wrapping around the grid (toroidal).
    fn count_alive_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        for i in -1..=1 {
            for j in -1..=1 {
                if i == 0 && j == 0 {
                    continue; // Skip self
                }
                // Modulo arithmetic for toroidal wrap-around
                let neighbor_x = (x as isize + i + WORLD_SIZE as isize) as usize % WORLD_SIZE;
                let neighbor_y = (y as isize + j + WORLD_SIZE as isize) as usize % WORLD_SIZE;

                if self.world[neighbor_x][neighbor_y] > 0 {
                    // Check age > 0
                    count += 1;
                }
            }
        }
        count
    }
}
