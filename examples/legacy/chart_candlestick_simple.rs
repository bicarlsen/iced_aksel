//! Chart shape layer example
//!
//! A refactored, single-file version of the candlestick chart application.
//! Now includes a separate, synchronized volume panel.

use std::{
    collections::BTreeMap,
    ops::{RangeInclusive, Sub},
};

use chrono::TimeZone;
use iced::{
    Color, Element, Task, Theme,
    widget::{Container, column, container, text},
};
use iced_extras::widget::chart::{
    self, Axis, Chart, DragDelta, Layer, PlotPoint, PlotRectangle, Position, State,
    axis::TickLine,
    render::Buffer,
    scale::{Linear, Tick},
    shape::{self, Length, Rectangle},
};

// --- Constants ---

const X_AXIS_ID: AxisId = "x_axis";
const Y_AXIS_ID: AxisId = "y_axis";
const Y_VOL_AXIS_ID: AxisId = "y_vol_axis";

// --- Shared Types ---

/// Type alias for the unique ID of a chart axis.
pub type AxisId = &'static str;

/// Defines the messages that can be sent to update the application state.
#[derive(Debug, Clone)]
enum Message {
    /// A message to trigger a full rebuild of the chart layers.
    UpdateChart,
    /// A message sent when the main plot area is dragged.
    OnPlotDrag(DragDelta),
    /// A message sent when an axis is dragged (for zooming).
    OnAxisDrag(AxisId, f32),
}

// --- Main Entry Point ---

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application Structure ---

/// Represents the top-level state of the iced application.
struct ExampleApp {
    candlestick_chart: CandlestickChart,
}

impl ExampleApp {
    /// Initializes the application and its component.
    fn init() -> (Self, Task<Message>) {
        let app = Self {
            candlestick_chart: CandlestickChart::new(),
        };

        // Start with an `UpdateChart` message to trigger the first layer build.
        (app, Task::done(Message::UpdateChart))
    }

    /// Handles messages and updates the application state.
    fn update(&mut self, message: Message) -> Task<Message> {
        // Delegate message handling to the chart component.
        self.candlestick_chart.handle_message(message);

        // No asynchronous tasks are needed in response to updates.
        Task::none()
    }

    /// Renders the application's view.
    fn view(&self) -> Element<'_, Message> {
        // The candlestick chart component now returns the complete
        // single chart (price + volume layers).
        let chart = self
            .candlestick_chart
            .view()
            .on_drag(Message::OnPlotDrag)
            .on_axis_drag(Message::OnAxisDrag);

        // Display the chart column.
        column![chart].into()
    }

    /// Defines the application's theme.
    const fn theme(&self) -> Theme {
        Theme::Dark
    }

    /// Runs the iced application.
    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .antialiasing(true)
            .run()
    }
}

// --- Chart Component ---

/// Holds settings related to chart rendering and interaction.
#[derive(Debug, Clone, Copy)]
struct ChartSettings {
    /// Automatically adjust the Y-axis to fit visible data.
    y_lock: bool,
    /// Percentage of padding to add when `y_lock` is enabled (e.g., 0.05 = 5%).
    y_lock_margin: f32,
    /// The width of a candle as a percentage (0.0 to 1.0) of its available time interval.
    candle_width_ratio: f32,
    /// The fixed "width" of a single candle in plot coordinates (e.g., timestamp difference).
    time_interval: i64,
}

/// Manages the state and rendering of the candlestick and volume charts.
struct CandlestickChart {
    /// Configuration for chart behavior.
    settings: ChartSettings,
    /// The raw candle data, mapped by timestamp.
    data: BTreeMap<i64, Candle>,

    /// The single state managing all axes (X, Y-Price, Y-Volume).
    state: chart::State<AxisId>,

    /// Layer for price candlesticks (X -> Y_AXIS_ID).
    candle_layer: Layer<AxisId>,
    /// Layer for volume bars (X -> Y_VOL_AXIS_ID).
    volume_layer: Layer<AxisId>,
}

impl CandlestickChart {
    /// Creates a new `CandlestickChart` with default settings and data.
    pub fn new() -> Self {
        let candle_data = generate_candlestick_data();
        let time_interval = Self::find_time_interval(&candle_data).unwrap_or(1);
        let y_lock_margin = 0.05;

        // --- Define Initial Ranges ---
        let initial_x_range = (0.0, 100.0);
        let initial_y_range = get_y_range_with_margin(&candle_data, initial_x_range, y_lock_margin)
            .unwrap_or((0.0, 100.0));
        let initial_y_vol_range = get_y_vol_range_with_margin(&candle_data, initial_x_range, 0.1) // 10% margin for volume
            .unwrap_or((0.0, 1000.0));

        // --- Create Single Chart State ---
        let mut state: State<AxisId> = chart::State::new();
        // Add all three axes to the same state
        state.set_axis(X_AXIS_ID, Self::create_x_axis(initial_x_range));
        state.set_axis(Y_AXIS_ID, Self::create_y_axis(initial_y_range));
        state.set_axis(Y_VOL_AXIS_ID, Self::create_vol_axis(initial_y_vol_range));

        Self {
            settings: ChartSettings {
                y_lock: true,
                y_lock_margin,
                candle_width_ratio: 0.5,
                time_interval,
            },
            data: candle_data,
            state,
            // Candle layer maps X -> Y_PRICE
            candle_layer: Layer::new(X_AXIS_ID, Y_AXIS_ID),
            // Volume layer maps X -> Y_VOLUME
            volume_layer: Layer::new(X_AXIS_ID, Y_VOL_AXIS_ID),
        }
    }

    /// Returns the single `Chart` widget containing all layers.
    pub fn view(&self) -> chart::Chart<'_, AxisId, Message> {
        Chart::new(&self.state)
            .layer(&self.candle_layer)
            .layer(&self.volume_layer)
    }

    /// Handles incoming messages and updates the chart state.
    pub fn handle_message(&mut self, message: Message) {
        match message {
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
        }
    }

    // --- Event Handlers ---

    /// Logic for panning the chart.
    fn handle_plot_drag(&mut self, delta: DragDelta) {
        // --- Pan X-Axis ---
        self.state
            .get_axis_mut(&X_AXIS_ID)
            .expect("X-axis must exist")
            .scale_mut()
            .pan(delta.x);

        // --- Pan Y-Axes (Conditionally) ---

        // Only pan the Price Y-axis if y_lock is off.
        if !self.settings.y_lock {
            self.state
                .get_axis_mut(&Y_AXIS_ID)
                .expect("Price Y-axis must exist")
                .scale_mut()
                .pan(delta.y);
        }

        // Volume Y-axis always pans, as it's not tied to y_lock.
        self.state
            .get_axis_mut(&Y_VOL_AXIS_ID)
            .expect("Volume Y-axis must exist")
            .scale_mut()
            .pan(delta.y);

        // Derived domain updates are now handled in rebuild_layers()
    }

    /// Logic for zooming the chart by dragging an axis.
    fn handle_axis_drag(&mut self, id: AxisId, delta: f32) {
        // Zoom factor. `delta` is small, so `* 2.0` increases sensitivity.
        let factor = 1.0 + delta * 2.0;

        match id {
            X_AXIS_ID => {
                // --- Zoom X-Axis ---
                let anchor = Some(1.0);
                self.state
                    .get_axis_mut(&X_AXIS_ID)
                    .expect("X-axis must exist")
                    .scale_mut()
                    .zoom(factor, anchor);
            }
            Y_AXIS_ID => {
                // --- Zoom Price Y-Axis ---
                self.state
                    .get_axis_mut(&Y_AXIS_ID)
                    .expect("Price Y-axis must exist")
                    .scale_mut()
                    .zoom(factor, Some(0.5));
                // Manually zooming the Y-axis disables y_lock.
                self.settings.y_lock = false;
            }
            Y_VOL_AXIS_ID => {
                // --- Zoom Volume Y-Axis ---
                self.state
                    .get_axis_mut(&Y_VOL_AXIS_ID)
                    .expect("Volume Y-axis must exist")
                    .scale_mut()
                    .zoom(factor, Some(0.5));
            }
            _ => {} // Unknown axis
        }
    }

    // --- Private Helpers ---

    /// Clears and rebuilds all shape layers based on the current data and view.
    fn rebuild_layers(&mut self) {
        // --- Pre-Render State Synchronization ---
        // First, update all derived domains before drawing.

        // If y_lock is on, recalculate the Y-axis domain to fit visible data.
        if self.settings.y_lock {
            self.update_y_lock_domain();
        }

        // Always update the volume domain to fit visible data.
        // (This could also be made conditional with its own lock setting).
        self.update_y_vol_domain();
        // --- End State Synchronization ---

        // Calculate the candle width in plot units (a fraction of the time interval)
        let candle_width_plot =
            self.settings.time_interval as f32 * self.settings.candle_width_ratio;
        let candle_width = Length::Plot(candle_width_plot);

        // Get the visible range from the single X-axis
        let x_domain = self
            .state
            .get_axis(&X_AXIS_ID)
            .expect("X-axis must exist")
            .scale()
            .domain();

        let x_range = (x_domain.0.floor() as i64)..=(x_domain.1.ceil() as i64);

        // Clear both layers
        self.candle_layer.clear();
        self.volume_layer.clear();

        // Iterate once and push shapes to their respective layers
        for (time, candle) in self.data.range(x_range) {
            // Add to price layer
            candle.add_to_layer(time, candle_width, self.candle_layer.buffer_mut());
            // Add to volume layer
            candle.add_volume_to_layer(time, candle_width, self.volume_layer.buffer_mut());
        }
    }

    /// Finds the smallest time difference between consecutive data points.
    fn find_time_interval(data: &BTreeMap<i64, Candle>) -> Option<i64> {
        data.keys()
            .skip(1)
            .zip(data.keys())
            .map(|(next, prev)| next - prev)
            .min()
    }

    /// Recalculates and sets the Price Y-axis domain to fit the visible data.
    fn update_y_lock_domain(&mut self) {
        let x_domain = self
            .state
            .get_axis(&X_AXIS_ID)
            .expect("X-axis must exist")
            .scale()
            .domain();

        let new_y_range = get_y_range_with_margin(
            &self.data,
            (x_domain.0, x_domain.1),
            self.settings.y_lock_margin,
        );

        if let Some((min, max)) = new_y_range {
            self.state
                .get_axis_mut(&Y_AXIS_ID)
                .expect("Price Y-axis must exist")
                .scale_mut()
                .set_domain(min, max);
        }
    }

    /// Recalculates and sets the Volume Y-axis domain to fit the visible data.
    fn update_y_vol_domain(&mut self) {
        let x_domain = self
            .state
            .get_axis(&X_AXIS_ID)
            .expect("X-axis must exist")
            .scale()
            .domain();

        let new_y_vol_range =
            get_y_vol_range_with_margin(&self.data, (x_domain.0, x_domain.1), 0.1); // 10% margin

        if let Some((min, max)) = new_y_vol_range {
            self.state
                .get_axis_mut(&Y_VOL_AXIS_ID)
                .expect("Volume Y-axis must exist")
                .scale_mut()
                .set_domain(min, max);
        }
    }

    /// Factory for creating the main X-axis (with labels).
    fn create_x_axis(range: (f32, f32)) -> Axis {
        let scale = Linear::new(range.0, range.1);

        let tick_renderer = |t: Tick<f32>| -> Option<TickLine> {
            let timestamp_seconds = t.value as i64 * 60; // Assuming 1 unit = 1 minute
            let datetime = chrono::Utc.timestamp_opt(timestamp_seconds, 0).single()?;
            let text = datetime.format("%H:%M").to_string();
            Some(TickLine::simple(Color::WHITE, &text))
        };

        Axis::new(scale, Position::Bottom).with_tick_renderer(Box::new(tick_renderer))
    }

    /// Factory for creating the Price Y-axis.
    fn create_y_axis(range: (f32, f32)) -> Axis {
        let scale = Linear::new(range.0, range.1);
        Axis::new(scale, Position::Right)
    }

    /// Factory for creating the Volume Y-axis.
    fn create_vol_axis(range: (f32, f32)) -> Axis {
        let scale = Linear::new(range.0, range.1);
        // Place on the right as well, as requested. The library should handle stacking them.
        Axis::new(scale, Position::Right)
    }
}

// --- Data Structures & Utilities ---

/// Represents a single candlestick.
#[derive(Debug, Clone, Copy)]
struct Candle {
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
}

impl Candle {
    /// Creates a new `Candle`.
    pub fn new(open: f32, high: f32, low: f32, close: f32, volume: f32) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// The color of the candle, based on its open and close prices.
    fn color(&self) -> Color {
        if self.close > self.open {
            Color::from_rgb8(69, 170, 153) // Bullish (Green)
        } else if self.close < self.open {
            Color::from_rgb8(232, 101, 111) // Bearish (Red)
        } else {
            Color::from_rgb8(150, 150, 150) // Neutral (Gray)
        }
    }

    fn volume_color(&self) -> Color {
        if self.close > self.open {
            Color::from_rgba8(69, 170, 153, 0.5) // Bullish (Green)
        } else if self.close < self.open {
            Color::from_rgba8(232, 101, 111, 0.5) // Bearish (Red)
        } else {
            Color::from_rgba8(150, 150, 150, 0.5) // Neutral (Gray)
        }
    }

    // --- Price Drawing ---

    fn body_height(&self) -> Length {
        Length::Plot(self.open.sub(self.close).abs())
    }

    fn wick_width(&self) -> Length {
        Length::Screen(1.0)
    }

    fn wick_height(&self) -> Length {
        Length::Plot(self.high.sub(self.low))
    }

    /// Draws this candle's shapes (wick and body) into a render buffer.
    pub fn add_to_layer(&self, x: &i64, width: Length, buffer: &mut Buffer) {
        let color = self.color();
        let x_position = *x as f32;

        let wick_y_center = (self.high + self.low) / 2.0;
        let wick = Rectangle {
            position: PlotPoint::new(x_position, wick_y_center),
            h_anchor: shape::HorizontalOrientation::Center,
            v_anchor: shape::VerticalOrientation::Center,
            width: self.wick_width(),
            height: self.wick_height(),
            fill: Some(color),
            stroke: None,
        };

        let body_y_center = (self.open + self.close) / 2.0;
        let body = Rectangle {
            position: PlotPoint::new(x_position, body_y_center),
            h_anchor: shape::HorizontalOrientation::Center,
            v_anchor: shape::VerticalOrientation::Center,
            width, // Use the dynamic width passed in
            height: self.body_height(),
            fill: Some(color),
            stroke: None,
        };

        buffer.push(wick);
        buffer.push(body);
    }

    // --- Volume Drawing ---

    /// Draws this candle's volume bar into a render buffer.
    pub fn add_volume_to_layer(&self, x: &i64, width: Length, buffer: &mut Buffer) {
        let color = self.volume_color();
        let x_position = *x as f32;

        let relative_volume = self.volume / 10.;

        // Volume bars are drawn from the bottom (0) up to the volume value.
        // We position the center of the bar at `volume / 2.0`.
        let bar = Rectangle {
            position: PlotPoint::new(x_position, relative_volume / 2.0),
            h_anchor: shape::HorizontalOrientation::Center,
            v_anchor: shape::VerticalOrientation::Center,
            width, // Use the same width as the candle body
            height: Length::Plot(relative_volume),
            fill: Some(color),
            stroke: None,
        };

        buffer.push(bar);
    }
}

/// Generates a set of pseudo-random candlestick data.
fn generate_candlestick_data() -> BTreeMap<i64, Candle> {
    let mut data = BTreeMap::new();
    let mut previous_close = 100.0 + rand::random::<f32>() * 20.0;

    for i in 0..1_000 {
        let open = previous_close;
        let high_delta = rand::random::<f32>() * 10.0;
        let low_delta = rand::random::<f32>() * -10.0;
        let mut high = open + high_delta;
        let mut low = open + low_delta;

        if high < low {
            std::mem::swap(&mut high, &mut low);
        }

        let close = low + (high - low) * rand::random::<f32>();

        data.insert(
            i as i64,
            Candle {
                open,
                high,
                low,
                close,
                volume: 1_000_000.0 + rand::random::<f32>() * 5_000_000.0, // More realistic volume
            },
        );

        previous_close = close;
    }
    data
}

// --- Price Data Helpers ---

/// Finds the min (low) and max (high) prices within a given x-range.
fn get_visible_y_range(
    data: &BTreeMap<i64, Candle>,
    x_range: RangeInclusive<i64>,
) -> Option<(f32, f32)> {
    let visible_candles = data.range(x_range);

    let y_low = visible_candles
        .clone()
        .map(|(_key, candle)| candle.low)
        .min_by(|a, b| a.total_cmp(b));

    let y_high = visible_candles
        .map(|(_key, candle)| candle.high)
        .max_by(|a, b| a.total_cmp(b));

    y_low.zip(y_high)
}

/// Calculates a Y-domain (min, max) for the price chart.
fn get_y_range_with_margin(
    data: &BTreeMap<i64, Candle>,
    x_domain: (f32, f32),
    margin_percent: f32,
) -> Option<(f32, f32)> {
    let x_range = (x_domain.0.floor() as i64)..=(x_domain.1.ceil() as i64);

    get_visible_y_range(data, x_range).map(|(y_low, y_high)| {
        let margin_amount = (y_high - y_low + 1e-6) * margin_percent;
        (y_low - margin_amount, y_high + margin_amount)
    })
}

// --- Volume Data Helpers ---

/// Finds the max volume within a given x-range.
fn get_visible_y_vol_range(
    data: &BTreeMap<i64, Candle>,
    x_range: RangeInclusive<i64>,
) -> Option<f32> {
    data.range(x_range)
        .map(|(_key, candle)| candle.volume)
        .max_by(|a, b| a.total_cmp(b))
}

/// Calculates a Y-domain (0, max) for the volume chart.
fn get_y_vol_range_with_margin(
    data: &BTreeMap<i64, Candle>,
    x_domain: (f32, f32),
    margin_percent: f32,
) -> Option<(f32, f32)> {
    let x_range = (x_domain.0.floor() as i64)..=(x_domain.1.ceil() as i64);

    get_visible_y_vol_range(data, x_range).map(|y_high| {
        let margin_amount = (y_high + 1e-6) * margin_percent;
        (0.0, y_high + margin_amount) // Volume axis always starts at 0
    })
}
