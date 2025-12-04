use aksel::{PlotPoint, Scale, scale::Linear};
use iced::{
    Color, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Length, State,
    axis::{self, GridLine, TickLine},
    plot::{Items, Plot},
};
use std::collections::HashMap;

// Import shapes
use iced_aksel::Stroke;
use iced_aksel::shape::{Label, Polygon, Polyline, Rectangle};

type AxisId = String;

// =========================================================
//  Line Series (The Renderable Item)
// =========================================================

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub name: String,
    pub values: Vec<f64>, // Implicit X based on index

    // Axis Binding
    pub y_key: String,

    // Appearance
    pub color: Color,
    pub width: f32,
    pub show_markers: bool,
    pub fill_color: Option<Color>,
}

impl LineSeries {
    /// Creates a new series.
    /// By default, binds to the standard "Y" axis.
    pub fn new(name: impl Into<String>, color: Color) -> Self {
        Self {
            name: name.into(),
            values: Vec::new(),
            y_key: "Y".to_string(),
            color,
            width: 2.0,
            show_markers: false,
            fill_color: None,
        }
    }

    /// Binds this series to a specific Y-axis (e.g., "Temperature", "Pressure").
    pub fn axis(mut self, y_id: impl Into<String>) -> Self {
        self.y_key = y_id.into();
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn markers(mut self, show: bool) -> Self {
        self.show_markers = show;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    // --- Data Methods ---

    pub fn push(mut self, value: f64) -> Self {
        self.values.push(value);
        self
    }

    pub fn extend(mut self, values: impl IntoIterator<Item = f64>) -> Self {
        self.values.extend(values);
        self
    }
}

// Implement Items for LineSeries so it can draw itself
impl Items<f64> for LineSeries {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, _theme: &Theme) {
        if self.values.len() < 2 {
            return;
        }

        // 1. Prepare Points
        // X is implicitly the index (0.0, 1.0, 2.0...)
        let points: Vec<PlotPoint<f64>> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &v)| PlotPoint::new(i as f64, v))
            .collect();

        // 2. Draw Fill
        if let Some(fill_color) = self.fill_color {
            if let Some(first) = points.first() {
                if let Some(last) = points.last() {
                    // Find visual floor (min value) to close the polygon
                    let min_y = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));

                    let mut fill_points = points.clone();
                    fill_points.push(PlotPoint::new(last.x, min_y));
                    fill_points.push(PlotPoint::new(first.x, min_y));

                    plot.add_shape(Polygon::new(fill_points).fill(fill_color));
                }
            }
        }

        // 3. Draw Stroke
        plot.add_shape(Polyline {
            points: points.clone(),
            stroke: Stroke::new(self.color, Length::Screen(self.width)),
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 10.0,
        });

        // 4. Draw Markers
        if self.show_markers {
            for point in points {
                let marker_size = Length::Screen(self.width * 2.5);
                plot.add_shape(Rectangle::new(point, marker_size, marker_size).fill(self.color));
            }
        }
    }
}

// =========================================================
//  Line Chart (The Orchestrator)
// =========================================================

pub struct LineChart {
    state: State<AxisId, f64>,
    series: Vec<LineSeries>,
    // The chart maintains the list of known axes to avoid recreating them
    defined_axes: Vec<String>,
}

impl LineChart {
    pub const X: &'static str = "X";
    pub const Y: &'static str = "Y";

    pub fn new() -> Self {
        Self {
            state: State::new(),
            series: Vec::new(),
            defined_axes: Vec::new(),
        }
    }

    /// Pre-registers standard axes.
    pub fn with_default_axes(mut self) -> Self {
        self.with_axis(
            Self::X,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom),
        );
        self.with_axis(Self::Y, y_axis(0.0, 1.0));
        self
    }

    // =========================================================
    //  Configuration
    // =========================================================

    /// Configure a custom axis (e.g. "Temperature" on the Right).
    pub fn with_axis(&mut self, id: impl Into<String>, axis: Axis<f64>) {
        let key = id.into();
        self.state.set_axis(key.clone(), axis);
        if !self.defined_axes.contains(&key) {
            self.defined_axes.push(key);
        }
        self.auto_scale();
    }

    /// Adds a series. Ensures its required Y-axis exists.
    pub fn push_series(&mut self, series: LineSeries) {
        self.ensure_axes_exist(&series);
        self.series.push(series);
        self.auto_scale();
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.auto_scale();
    }

    pub fn get_last(&self) -> Option<&LineSeries> {
        self.series.last()
    }

    // =========================================================
    //  Data Injection
    // =========================================================

    /// Pushes a Label + Value pair.
    /// The label is currently ignored (future proofing), but the API is stable.
    /// The value is added to the LAST series (or a default one is created).
    pub fn push(&mut self, _label: impl Into<String>, value: f64) {
        // Create default series if empty
        if self.series.is_empty() {
            let default_series = LineSeries::new("Series 1", Color::from_rgb(0.2, 0.4, 0.8));
            self.ensure_axes_exist(&default_series);
            self.series.push(default_series);
        }

        if let Some(last) = self.series.last_mut() {
            last.values.push(value);
        }

        self.auto_scale();
    }

    pub fn push_value(&mut self, value: f64) {
        self.push("", value);
    }

    pub fn push_to(&mut self, index: usize, _label: impl Into<String>, value: f64) {
        if let Some(series) = self.series.get_mut(index) {
            series.values.push(value);
            self.auto_scale();
        }
    }

    pub fn push_value_to(&mut self, index: usize, value: f64) {
        self.push_to(index, "", value);
    }

    // Helper for main.rs compatibility
    pub fn push_value_last_series(&mut self, value: f64) {
        self.push_value(value);
    }

    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    // =========================================================
    //  Internal Logic
    // =========================================================

    fn ensure_axes_exist(&mut self, series: &LineSeries) {
        // 1. Ensure Global X exists
        let x_key = Self::X.to_string();
        if !self.defined_axes.contains(&x_key) {
            self.state.set_axis(
                x_key.clone(),
                Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom),
            );
            self.defined_axes.push(x_key);
        }

        // 2. Ensure Series-Specific Y exists
        if !self.defined_axes.contains(&series.y_key) {
            self.state.set_axis(series.y_key.clone(), y_axis(0., 1.));
            self.defined_axes.push(series.y_key.clone());
        }
    }

    fn auto_scale(&mut self) {
        if self.series.is_empty() {
            return;
        }

        // 1. Scale Global X-Axis
        let max_len = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0);
        let x_max = (max_len as f64 - 1.0).max(0.0);

        let x_key = Self::X.to_string();
        if let Some(x_axis) = self.state.get_axis_mut(&x_key) {
            x_axis.scale_mut().set_domain(0.0, x_max);
        }

        // 2. Scale Y-Axes (Grouped by key)
        let mut y_bounds: HashMap<String, (f64, f64)> = HashMap::new();

        for s in &self.series {
            if s.values.is_empty() {
                continue;
            }

            let mut min = f64::MAX;
            let mut max = f64::MIN;
            for &v in &s.values {
                min = min.min(v);
                max = max.max(v);
            }

            let entry = y_bounds
                .entry(s.y_key.clone())
                .or_insert((f64::MAX, f64::MIN));
            entry.0 = entry.0.min(min);
            entry.1 = entry.1.max(max);
        }

        for (axis_id, (min, max)) in y_bounds {
            if let Some(axis) = self.state.get_axis_mut(&axis_id) {
                // 5% Padding
                let padding = if max > min { (max - min) * 0.05 } else { 1.0 };
                axis.scale_mut().set_domain(min, max + padding);
            }
        }
    }

    // =========================================================
    //  View
    // =========================================================

    pub fn chart<Message>(&self) -> Chart<'_, AxisId, f64, Message> {
        let mut chart = Chart::new(&self.state);

        for series in &self.series {
            // Layer series: X is always Global X, Y is series-specific Y
            chart = chart.layer(series, Self::X.to_string(), series.y_key.clone());
        }

        chart
    }
}

fn y_axis(min_y: f64, max_y: f64) -> Axis<f64> {
    Axis::new(Linear::new(min_y, max_y), axis::Position::Left).with_tick_renderer(|ctx| {
        match ctx.tick.level {
            0 => {
                let line = TickLine::simple(format!("{:.2}", ctx.tick.value));
                Some(line)
            }
            _ => None,
        }
    })
}
