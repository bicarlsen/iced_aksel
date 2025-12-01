//! Chart shape layer example

use std::{
    collections::{BTreeMap, btree_map},
    ops::Sub,
};

use chrono::TimeZone;
use iced::{
    Color, Element, Task, Theme,
    widget::{Slider, column, text},
};
use iced_extras::widget::chart::{
    self, Axis, Chart, DragDelta, Layer, PlotPoint, PlotRectangle, Position, Scale, State,
    axis::TickLine,
    render::Buffer,
    scale::{Linear, Tick},
    shape::{self, Length, Rectangle},
};

const X_ID: &str = "linear_x";
const Y_ID: &str = "linear_y";

type AxisId = &'static str;

#[derive(Debug, Clone)]
enum Message {
    UpdateChart,
    OnPlotDrag(DragDelta),
    OnAxisDrag(AxisId, f32),
}

struct ExampleApp {
    candlestick_chart: CandlestickChart,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let app = Self {
            candlestick_chart: CandlestickChart::init(),
        };

        // Start with 500 shapes so the app isn't empty
        (app, Task::done(Message::UpdateChart))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateChart => {
                self.candlestick_chart.rebuild_layers();
            }
            Message::OnPlotDrag(delta) => {
                self.candlestick_chart.on_plot_drag(delta);
                self.candlestick_chart.rebuild_layers();
            }
            Message::OnAxisDrag(id, delta) => {
                self.candlestick_chart.on_axis_drag(id, delta);
                self.candlestick_chart.rebuild_layers();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = self
            .candlestick_chart
            .as_chart()
            .on_drag(Message::OnPlotDrag)
            .on_axis_drag(Message::OnAxisDrag);

        column![chart].into()
    }

    const fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .antialiasing(true)
            .run()
    }
}

fn main() -> iced::Result {
    ExampleApp::run()
}

// Som udgangspunkt vil jeg gøre CandleStickChart agnostic til data, så meget som muligt. Den skal
// ændre sig og formere efter data, når den bliver bedt om det. Thats it
struct CandlestickChart {
    // Chart
    state: chart::State<AxisId>,

    // Settings
    y_lock: bool,
    y_lock_margin: f32, // Percentage of screen to add top and bottom when locked

    candle_width: f32, // How much of the space allowed for a candle, should it fill?
    time_interval: i64, // Hvor meget "plads" tager ét candle i plot-enheder?

    // Data
    candle_data: BTreeMap<i64, Candle>,

    // Rendering
    candle_layer: Layer<AxisId>,
    volume_layer: Layer<AxisId>,
}

impl CandlestickChart {
    const X_AXIS_ID: AxisId = "x_axis";
    const Y_AXIS_ID: AxisId = "y_axis";

    fn init() -> Self {
        // 1. Set default state
        let mut state: State<AxisId> = chart::State::new();
        let y_lock_margin = 0.05;

        let candle_data = generate_candlestick_data();

        let y_range = get_y_range(&candle_data, (0.0, 100.0), y_lock_margin);

        // Find det mindste interval mellem datapunkter
        // Dette antager, at dine data er regelmæssigt fordelt
        let time_interval = candle_data
            .keys()
            .skip(1)
            .zip(candle_data.keys())
            .map(|(next, prev)| next - prev)
            .min() // Find det mindste interval
            .unwrap_or(1); // Default til 1, hvis der er 0 eller 1 datapunkt

        // We want linear scales on both sides
        let linear_scale_x = Linear::new(0.0, 100.0);
        let linear_scale_y = Linear::new(y_range.0, y_range.1);

        // Set the axis for state to handle
        // X-AXIS
        // TODO: Overvej om vi skal lave definitionen af hvornår en tick skal udregnes, være op til `Axis` istedet for `Scale`?
        // For en hvilken som helst bruger der bare hurtigt vil bruge denne API, vil det være lidt tedious
        // at skulle implement sin helt egen `Scale` for at kunne bestemme at ticks skal være på halve tal istedet for hele f.eks.
        // Kan også overveje om det skal være extensions som er på scale::Linear i sig selv?
        // TODO: Hvad er f32? Skal vi definere PlotX(f32) f.eks?
        let tick_renderer_x = |t: Tick<f32>| -> Option<TickLine> {
            // 1. Konvertér f32-værdien tilbage til et i64 timestamp
            let timestamp_seconds = t.value as i64 * 60;

            // 2. Brug chrono til at lave et DateTime-objekt (i UTC)
            let datetime = chrono::Utc.timestamp_opt(timestamp_seconds, 0).single()?;

            // 3. Formatér tiden som "HH:MM"
            let text = datetime.format("%H:%M").to_string();

            // 4. Returner den færdige TickLine
            Some(TickLine::simple(Color::WHITE, &text))
        };
        let axis_x = Axis::new(linear_scale_x, Position::Bottom)
            .with_tick_renderer(Box::new(tick_renderer_x));

        // Y-AXIS
        let axis_y = Axis::new(linear_scale_y, Position::Right);

        state.set_axis(Self::X_AXIS_ID, axis_x);
        state.set_axis(Self::Y_AXIS_ID, axis_y);

        Self {
            state,
            candle_layer: Layer::new(Self::X_AXIS_ID, Self::Y_AXIS_ID),
            volume_layer: Layer::new(Self::X_AXIS_ID, Self::Y_AXIS_ID),
            y_lock: true,
            y_lock_margin,
            candle_width: 0.5,
            time_interval,
            candle_data,
        }
    }

    // GETTERS
    fn get_x_axis(&self) -> &Axis {
        self.state
            .get_axis(&Self::X_AXIS_ID)
            .expect("Should always be there")
    }

    fn get_x_axis_mut(&mut self) -> &mut Axis {
        self.state
            .get_axis_mut(&Self::X_AXIS_ID)
            .expect("Should always be there")
    }

    fn get_y_axis(&self) -> &Axis {
        self.state
            .get_axis(&Self::Y_AXIS_ID)
            .expect("Should always be there")
    }

    fn get_y_axis_mut(&mut self) -> &mut Axis {
        self.state
            .get_axis_mut(&Self::Y_AXIS_ID)
            .expect("Should always be there")
    }

    fn get_candles_in_range_x(&self) -> btree_map::Range<i64, Candle> {
        self.candle_data.range(
            self.get_x_axis().scale().domain().0 as i64
                ..=self.get_x_axis().scale().domain().1 as i64,
        )
    }

    fn get_plot_rectangle(&self) -> PlotRectangle {
        self.state
            .get_scales_plotbounds(Self::X_AXIS_ID, Self::Y_AXIS_ID)
            .expect("There should ALWAYS be a bounds accessible for CandlestickChart")
    }

    // SETTERS
    fn set_y_lock(&mut self, lock: bool) {
        self.y_lock = lock;
    }

    fn rebuild_layers(&mut self) {
        let bounds = self.get_plot_rectangle();

        self.candle_layer.clear();

        let plot_width = self.time_interval as f32 * self.candle_width;

        // 2. Opret en Længde-type, der skalerer med x-aksen.
        let candle_body_width = Length::Plot(plot_width);
        for (time, candle) in self
            .candle_data
            .range(bounds.min_x() as i64..=bounds.max_x() as i64)
        {
            // Send den beregnede, dynamiske bredde med
            candle.add_to_layer(time, candle_body_width, self.candle_layer.buffer_mut());
        }
    }

    // EVENTS
    fn on_plot_drag(&mut self, delta: DragDelta) {
        // Just pan x-axis as is
        // TODO: Consider axis.pan() instead of axis.scale_mut().pan()
        self.get_x_axis_mut().scale_mut().pan(delta.x);
        self.get_y_axis_mut().scale_mut().pan(delta.y);

        if self.y_lock {
            // Handle y-axis lock behavior
            let y_low = self
                .get_candles_in_range_x()
                .map(|(_timestamp, candle)| candle.low)
                .min_by(|a, b| a.total_cmp(b));

            let y_high = self
                .get_candles_in_range_x()
                .map(|(_timestamp, candle)| candle.high)
                .max_by(|a, b| a.total_cmp(b));

            if let Some((y_low, y_high)) = y_low.zip(y_high) {
                let margin = (y_high - y_low) * self.y_lock_margin;
                self.get_y_axis_mut()
                    .scale_mut()
                    .set_domain(y_low - margin, y_high + margin);
            }
        }
    }

    fn on_axis_drag(&mut self, id: AxisId, delta: f32) {
        let axis = self.state.get_axis(&id).expect("Should always have");

        match axis.orientation() {
            chart::Orientation::Horizontal => {
                let factor = 1.0 + delta * 2.0;
                self.get_x_axis_mut().scale_mut().zoom(factor, Some(1.));
            }
            chart::Orientation::Vertical => {
                let factor = 1.0 + delta * 2.0;

                // TODO: Some(ancher). Could maybe just be forced?
                self.get_y_axis_mut().scale_mut().zoom(factor, Some(0.5));
                self.y_lock = false;
            }
        }
    }

    fn as_chart(&self) -> chart::Chart<'_, AxisId, Message> {
        Chart::new(&self.state)
            .layer(&self.candle_layer)
            .layer(&self.volume_layer)
    }
}

#[derive(Debug, Clone, Copy)]
struct Candle {
    open: f32,
    high: f32,
    low: f32,
    close: f32,
    volume: f32,
}

impl Candle {
    const WICK_WIDTH: f32 = 0.1; // Smaller width for the wick

    fn new(open: f32, high: f32, low: f32, close: f32, volume: f32) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume,
        }
    }

    fn body_height(&self) -> Length {
        Length::Plot(self.open.sub(self.close).abs())
    }

    fn wick_width(&self) -> Length {
        Length::Screen(1.)
    }

    fn wick_height(&self) -> Length {
        Length::Plot(self.high.sub(self.low))
    }

    /// --- THIS IS THE UPDATED FUNCTION ---
    fn add_to_layer(&self, x: &i64, width: Length, buffer: &mut Buffer) {
        // 1. Determine color for this candle
        let color = if self.close > self.open {
            Color::from_rgb8(69, 170, 153) // Bullish
        } else if self.close < self.open {
            Color::from_rgb8(232, 101, 111) // Bearish
        } else {
            Color::from_rgb8(232, 101, 111) // Neutral
        };

        // 2. --- Create the Wick ---
        // The wick is one line from high to low.
        let wick_y_center = (self.high + self.low) / 2.0;

        let wick = Rectangle {
            position: PlotPoint::new(*x as f32, wick_y_center),
            h_anchor: shape::HorizontalOrientation::Center,
            v_anchor: shape::VerticalOrientation::Center,
            width: self.wick_width(),
            height: self.wick_height(),
            fill: Some(color), // Use the same color
            stroke: None,
        };

        // 3. --- Create the Body ---
        // This is the same code as before.
        let body_y_center = (self.open + self.close) / 2.0;

        let body = Rectangle {
            position: PlotPoint::new(*x as f32, body_y_center),
            h_anchor: shape::HorizontalOrientation::Center,
            v_anchor: shape::VerticalOrientation::Center,
            width,
            height: self.body_height(),
            fill: Some(color), // Use the same color
            stroke: None,
        };

        // 4. --- Push to buffer ---
        // Push the wick FIRST, so the body draws on top.
        buffer.push(wick);
        buffer.push(body);
    }
}

fn get_y_range(
    candle_data: &BTreeMap<i64, Candle>,
    x_range: (f32, f32),
    margin: f32,
) -> (f32, f32) {
    // 1. Define the key range.
    // I'm using `..=` (inclusive) instead of `..` (exclusive)
    // so that the candle at `x_range.1` is included in the calculation.
    let key_range = (x_range.0 as i64)..=(x_range.1 as i64);

    // 2. Get the min low price *within the key_range*.
    // FIX: The closure must be |(_key, candle)| ...
    let y_low = candle_data
        .range(key_range.clone()) // Use the key_range
        .map(|(_key, candle)| candle.low)
        .min_by(|a, b| a.total_cmp(b));

    // 3. Get the max high price *within the key_range*.
    // FIX: Use the key_range here, not candle_data.values().
    let y_high = candle_data
        .range(key_range) // Use the key_range
        .map(|(_key, candle)| candle.high)
        .max_by(|a, b| a.total_cmp(b));

    // 4. Calculate the final range with margin.
    // Your .zip() logic was already correct for handling `Option`.
    if let Some((y_low, y_high)) = y_low.zip(y_high) {
        // Renamed `margin` to `margin_amount` for clarity
        let margin_amount = (y_high - y_low) * margin;
        (y_low - margin_amount, y_high + margin_amount)
    } else {
        // Fallback if the range was empty
        (0.0, 100.0)
    }
}

fn generate_candlestick_data() -> BTreeMap<i64, Candle> {
    let mut data = BTreeMap::new();

    // 1. Start with an initial "previous close" price for the very first candle
    let mut previous_close = rand::random::<f32>() * 100.0;

    for i in 0..1_000 {
        // 2. The new open price is the previous candle's close price
        let open = previous_close;

        // 3. Generate high, low, and close based on this open price
        let high = open + rand::random::<f32>() * 10.0; // At least as high as open
        let low = open - rand::random::<f32>() * 10.0; // At least as low as open
        let close = low + (high - low) * rand::random::<f32>(); // Close is between low and high

        data.insert(
            i as i64,
            Candle {
                open,
                high,
                low,
                close,
                volume: 0.0,
            },
        );

        // 4. IMPORTANT: Update previous_close for the *next* iteration
        previous_close = close;
    }
    data
}
