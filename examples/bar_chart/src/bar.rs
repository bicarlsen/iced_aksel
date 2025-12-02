use aksel::{Scale, scale::Linear};
use iced::{Color, Theme};
use iced_aksel::{
    Axis, Chart, Length, State,
    axis::{self, TickLine},
    plot::{Items, Plot},
    shape::Rectangle,
};

const X_ID: &str = "linear_x";
const Y_ID: &str = "linear_y";

type AxisId = &'static str;

#[derive(Debug, Clone)]
pub struct BarData {
    pub label: String,
    pub value: f64,
}

impl From<(String, f64)> for BarData {
    fn from(data: (String, f64)) -> Self {
        BarData {
            label: data.0,
            value: data.1,
        }
    }
}

pub struct BarChart {
    // State is owned here, making this a self-contained widget
    pub state: State<AxisId, f64>,
    pub data: Vec<BarData>,
    pub bar_width: f64,
}

impl BarChart {
    pub fn new() -> Self {
        let mut state = State::new();

        // Standard Chart positioning (Bottom/Left)
        // Initialized with dummy range 0..1, auto_scale will fix this later
        state.set_axis(
            X_ID,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom),
        );
        state.set_axis(Y_ID, Axis::new(Linear::new(0.0, 1.0), axis::Position::Left));

        Self {
            state,
            data: Vec::new(),
            bar_width: 0.8, // Default nice width (relative to x-step of 1.0)
        }
    }

    /// Replaces the current data and re-calculates the axis scales
    pub fn refresh(&mut self) {
        self.auto_scale();
    }

    /// Returns the Chart widget.
    ///
    /// We return 'Chart' instead of 'Element' so the user can chain
    /// .on_drag(), .width(), or .height() before calling .into().
    pub fn view<'a, Message>(&'a self) -> Chart<'_, AxisId, f64, Message> {
        Chart::new(&self.state).layer(self, X_ID, Y_ID)
    }

    /// Internal helper to fit the axes to the data
    fn auto_scale(&mut self) {
        let count = self.data.len() as f64;

        // Find max Y value (or default to 10.0 if empty to avoid 0 range)
        let max_value = self
            .data
            .iter()
            .map(|d| d.value)
            .fold(0.0, f64::max)
            .max(10.0);

        // Update X Axis: 0 to count (e.g., 5 items = 0..5)
        // We range from -0.5 to count-0.5 so the first (index 0) and last bars are fully visible
        if let Some(x_axis) = self.state.get_axis_mut(&X_ID) {
            *x_axis = Axis::new(Linear::new(-0.5, count - 0.5), axis::Position::Bottom);
        }

        // Update Y Axis: 0 to max_value * 1.1 (10% headroom)
        if let Some(y_axis) = self.state.get_axis_mut(&Y_ID) {
            *y_axis = Axis::new(Linear::new(0.0, max_value * 1.1), axis::Position::Left);
        }
    }

    /// Takes a normalized x value, returns nearest idx for data
    /// Will often be used to render the right x value
    fn get_nearest_x_idx(&self, normalized: f32) -> Option<usize> {
        let count = self.data.len();
        let idx = (normalized * count as f32).round() as usize;
        Some(idx.min(count - 1))
    }

    fn get_nearest_x_label(&self, normalized: f32) -> Option<String> {
        let idx = self.get_nearest_x_idx(normalized)?;
        self.data.get(idx).map(|d| d.label.clone())
    }
}

// --- The Drawing Logic ---

impl Items<f64> for BarChart {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        let palette = theme.palette();

        // Use the theme's primary color for the bars
        let bar_color = palette.primary;

        for (i, item) in self.data.iter().enumerate() {
            let x_index = i as f64;
            let val = item.value;

            // GEOMETRY CALCULATION:
            // Rectangle::new takes the CENTER.
            // To make a bar sit on the baseline (y=0) and go up to 'val':
            // Center Y = val / 2.0
            // Height   = val
            let center = aksel::PlotPoint::new(x_index, val / 2.0);

            let shape = Rectangle::new(center, Length::Plot(self.bar_width), Length::Plot(val))
                .fill(bar_color);

            plot.add_shape(shape);
        }
    }
}
