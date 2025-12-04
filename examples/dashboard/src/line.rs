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

type AxisId = &'static str;

// --- Data Structure ---

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
}

impl DataPoint {
    pub fn new(label: &'static str, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}

// =========================================================
//  Line Series (The Renderable Item)
// =========================================================

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub name: String,
    pub data: Vec<DataPoint>,

    // Axis Binding
    pub x_key: &'static str,
    pub y_key: &'static str,

    // Appearance
    pub color: Color,
    pub width: f32,
    pub show_markers: bool,
    pub fill_color: Option<Color>,
}

impl LineSeries {
    pub fn new(
        name: impl Into<String>,
        color: Color,
        x_key: &'static str,
        y_key: &'static str,
    ) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            // Default binding. If "X" or "Y" don't exist in State, chart will create them.
            x_key,
            y_key,
            color,
            width: 2.0,
            show_markers: false,
            fill_color: None,
        }
    }

    /// Binds this series to specific axes.
    pub fn axis(mut self, x_id: &'static str, y_id: &'static str) -> Self {
        self.x_key = x_id;
        self.y_key = y_id;
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

    pub fn push(mut self, label: &'static str, value: f64) -> Self {
        self.data.push(DataPoint::new(label, value));
        self
    }

    pub fn push_value(&mut self, value: f64) {
        self.data.push(DataPoint::new("", value));
    }
}

impl Items<f64> for LineSeries {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, _theme: &Theme) {
        if self.data.len() < 2 {
            return;
        }

        // 1. Prepare Points (X is Index)
        let points: Vec<PlotPoint<f64>> = self
            .data
            .iter()
            .enumerate()
            .map(|(i, p)| PlotPoint::new(i as f64, p.value))
            .collect();

        // 2. Draw Fill
        if let Some(fill_color) = self.fill_color {
            if let Some(first) = points.first() {
                if let Some(last) = points.last() {
                    let min_y = self
                        .data
                        .iter()
                        .map(|p| p.value)
                        .fold(f64::INFINITY, f64::min);

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
//  Line Chart (The Coordinator)
// =========================================================

pub struct LineChart {
    state: State<AxisId, f64>,
    series: Vec<LineSeries>,
}

impl LineChart {
    pub const X: &'static str = "X";
    pub const Y: &'static str = "Y";

    pub fn new() -> Self {
        Self {
            state: State::new(),
            series: Vec::new(),
        }
    }

    /// Pre-registers standard axes.
    pub fn with_default_axes(mut self) -> Self {
        self.with_axis(
            Self::X,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom),
        );
        self.with_axis(
            Self::Y,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Left),
        );
        self
    }

    // =========================================================
    //  Configuration
    // =========================================================

    /// Explicitly configure an axis in the State.
    /// This is how you "create" an axis manually.
    pub fn with_axis(&mut self, id: &'static str, axis: Axis<f64>) {
        self.state.set_axis(id, axis);
        self.auto_scale();
    }

    /// Adds a series. Ensures required axes exist in State.
    pub fn push_series(&mut self, series: LineSeries) {
        self.ensure_axes_exist(&series);
        self.series.push(series);
        self.auto_scale();
    }

    pub fn clear_series(&mut self) {
        self.series.clear();
        self.auto_scale();
    }

    // =========================================================
    //  Data Injection
    // =========================================================

    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    pub fn push_value_last_series(&mut self, value: f64) {
        if let Some(last_series) = self.series.last_mut() {
            last_series.push_value(value);
        }
    }

    // =========================================================
    //  Internal Logic
    // =========================================================

    /// Checks State for axes. If missing, creates defaults.
    fn ensure_axes_exist(&mut self, series: &LineSeries) {
        // Source of Truth: self.state.get_axis()

        if self.state.get_axis(&series.x_key).is_none() {
            self.state.set_axis(
                series.x_key.clone(),
                Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom),
            );
        }

        if self.state.get_axis(&series.y_key).is_none() {
            self.state.set_axis(
                series.y_key.clone(),
                Axis::new(Linear::new(0.0, 100.0), axis::Position::Left),
            );
        }

        // PRint axes
        println!("{:?}", self.state.axes());
    }

    fn auto_scale(&mut self) {
        if self.series.is_empty() {
            return;
        }

        let mut bounds: HashMap<&'static str, (f64, f64)> = HashMap::new();

        for s in &self.series {
            if s.data.is_empty() {
                continue;
            }

            let x_count = s.data.len() as f64;
            let s_min_x = 0.0;
            let s_max_x = (x_count - 1.0).max(0.0);

            let mut s_min_y = f64::MAX;
            let mut s_max_y = f64::MIN;
            for p in &s.data {
                s_min_y = s_min_y.min(p.value);
                s_max_y = s_max_y.max(p.value);
            }

            // Global Update
            let x_entry = bounds.entry(s.x_key.into()).or_insert((f64::MAX, f64::MIN));
            x_entry.0 = x_entry.0.min(s_min_x);
            x_entry.1 = x_entry.1.max(s_max_x);

            let y_entry = bounds.entry(s.y_key.into()).or_insert((f64::MAX, f64::MIN));
            y_entry.0 = y_entry.0.min(s_min_y);
            y_entry.1 = y_entry.1.max(s_max_y);
        }

        // Apply to Axes using State
        for (axis_id, (min, max)) in bounds {
            if let Some(axis) = self.state.get_axis_mut(&axis_id) {
                // Heuristic: Padding for Y, tight for X
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

        println!("{:?}", self.series);
        println!("{:?}", self.state);

        for series in &self.series {
            chart = chart.layer(series, series.x_key, series.y_key);
        }

        chart
    }
}
