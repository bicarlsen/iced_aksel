use aksel::{PlotPoint, Scale, scale::Linear};
use iced::{
    Color, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Length, State,
    axis::{self, TickLine},
    plot::{Items, Plot},
};

// Import shapes
use iced_aksel::Stroke;
use iced_aksel::shape::Polyline;
// Using Rectangle for markers as a robust fallback
use iced_aksel::shape::{Label, Rectangle};

type AxisId = &'static str;

#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
}

impl DataPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<(f64, f64)> for DataPoint {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

/// A simplified Line Chart focused on a single data series.
pub struct LineChart {
    state: State<AxisId, f64>,

    // Data
    data: Vec<DataPoint>,

    // Appearance
    color: Option<Color>,
    width: f32,
    show_markers: bool,

    // Configuration options
    fixed_x_range: Option<(f64, f64)>,
    fixed_y_range: Option<(f64, f64)>,
}

impl LineChart {
    const X_AXIS: &str = "X";
    const Y_AXIS: &str = "Y";

    /// Creates a new, empty Line Chart with default settings.
    pub fn new() -> Self {
        let mut state = State::new();

        // Initialize axes with a default 0.0-1.0 range
        state.set_axis(
            Self::X_AXIS,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom),
        );
        state.set_axis(
            Self::Y_AXIS,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Left).with_tick_renderer(|ctx| {
                match ctx.tick.level {
                    0 => Some(TickLine::simple(format!("{:.2}", ctx.tick.value))),
                    _ => None,
                }
            }),
        );

        Self {
            state,
            data: Vec::new(),

            // defaults
            color: None,
            width: 2.0,
            show_markers: false,

            fixed_x_range: None,
            fixed_y_range: None,
        }
    }

    // =========================================================
    //  Builder API (Configuration)
    //  Define the LOOK and BEHAVIOR here.
    // =========================================================

    /// Sets the color of the line. Uses Theme Primary if None.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the width of the line in pixels.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Toggles circular markers at each data point.
    pub fn markers(mut self, show: bool) -> Self {
        self.show_markers = show;
        self
    }

    /// Sets a fixed range for the X-axis (disables auto-scaling for X).
    pub fn x_range(mut self, min: f64, max: f64) -> Self {
        self.fixed_x_range = Some((min, max));
        self.update_scales();
        self
    }

    /// Sets a fixed range for the Y-axis (disables auto-scaling for Y).
    pub fn y_range(mut self, min: f64, max: f64) -> Self {
        self.fixed_y_range = Some((min, max));
        self.update_scales();
        self
    }

    // =========================================================
    //  Runtime Methods (Data Management)
    //  Call these in your `update` loop.
    // =========================================================

    /// Adds a new Y value. The X value is automatically calculated
    /// based on the current number of points (0.0, 1.0, 2.0...).
    pub fn push(&mut self, y: f64) {
        let x = self.data.len() as f64;
        self.data.push(DataPoint::new(x, y));
        self.update_scales();
    }

    /// Adds a specific (x, y) data point.
    pub fn push_point(&mut self, x: f64, y: f64) {
        self.data.push(DataPoint::new(x, y));
        self.update_scales();
    }

    /// Replaces the dataset entirely.
    pub fn set_data(&mut self, data: Vec<impl Into<DataPoint>>) {
        self.data = data.into_iter().map(|d| d.into()).collect();
        self.update_scales();
    }

    /// Clears all data points.
    pub fn clear(&mut self) {
        self.data.clear();
        self.update_scales();
    }

    // =========================================================
    //  Internal Logic
    // =========================================================

    fn update_scales(&mut self) {
        // 1. Determine X Range
        let (min_x, max_x) = if let Some(range) = self.fixed_x_range {
            range
        } else {
            self.calculate_auto_range(|p| p.x)
        };

        // 2. Determine Y Range
        let (min_y, max_y) = if let Some(range) = self.fixed_y_range {
            range
        } else {
            self.calculate_auto_range(|p| p.y)
        };

        // 3. Apply to Axes
        self.state
            .get_axis_mut(&Self::X_AXIS)
            .unwrap()
            .scale_mut()
            .set_domain(min_x, max_x);

        self.state
            .get_axis_mut(&Self::Y_AXIS)
            .unwrap()
            .scale_mut()
            .set_domain(min_y, max_y);
    }

    fn calculate_auto_range(&self, selector: impl Fn(&DataPoint) -> f64) -> (f64, f64) {
        if self.data.is_empty() {
            return (0.0, 1.0);
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for p in &self.data {
            let val = selector(p);
            min = min.min(val);
            max = max.max(val);
        }

        // Add 5% padding so lines don't hug the edges
        let padding = (max - min) * 0.05;
        let padding = if padding == 0.0 { 1.0 } else { padding };

        (min - padding, max + padding)
    }

    // --- View ---

    pub fn chart<Message>(&self) -> Chart<'_, AxisId, f64, Message> {
        Chart::new(&self.state).layer(self, Self::X_AXIS, Self::Y_AXIS)
    }
}

// --- Drawing Logic ---

impl Items<f64> for LineChart {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        if self.data.len() < 2 {
            return;
        }

        let palette = theme.palette();
        let line_color = self.color.unwrap_or(palette.primary);

        let points: Vec<PlotPoint<f64>> =
            self.data.iter().map(|p| PlotPoint::new(p.x, p.y)).collect();

        // Draw Line
        plot.add_shape(Polyline {
            points: points.clone(),
            stroke: Stroke::new(line_color, Length::Screen(self.width)),
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 10.0,
        });

        // Draw Markers
        if self.show_markers {
            for point in points {
                let marker_size = Length::Screen(self.width * 2.5);
                plot.add_shape(Rectangle::new(point, marker_size, marker_size).fill(line_color));
            }
        }
    }
}
