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
//  Shared Data Structure
// =========================================================

#[derive(Debug, Clone)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
}

impl DataPoint {
    pub fn new(label: impl Into<String>, value: f64) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}

// =========================================================
//  Line Series Implementation
// =========================================================

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub name: String,
    pub values: Vec<f64>,
    pub y_key: String,

    // Appearance
    pub color: Color,
    pub width: f32,
    pub show_markers: bool,
    pub fill_color: Option<Color>,
}

impl LineSeries {
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

    pub fn push(mut self, value: f64) -> Self {
        self.values.push(value);
        self
    }

    pub fn extend(mut self, values: impl IntoIterator<Item = f64>) -> Self {
        self.values.extend(values);
        self
    }
}

impl Items<f64> for LineSeries {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, _theme: &Theme) {
        if self.values.len() < 2 {
            return;
        }

        let points: Vec<PlotPoint<f64>> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &v)| PlotPoint::new(i as f64, v))
            .collect();

        // Draw Fill
        if let Some(fill_color) = self.fill_color {
            if let Some(first) = points.first() {
                if let Some(last) = points.last() {
                    let min_y = self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let mut fill_points = points.clone();
                    fill_points.push(PlotPoint::new(last.x, min_y));
                    fill_points.push(PlotPoint::new(first.x, min_y));
                    plot.add_shape(Polygon::new(fill_points).fill(fill_color));
                }
            }
        }

        // Draw Stroke
        plot.add_shape(Polyline {
            points: points.clone(),
            stroke: Stroke::new(self.color, Length::Screen(self.width)),
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
                plot.add_shape(Rectangle::new(point, marker_size, marker_size).fill(self.color));
            }
        }
    }
}

// =========================================================
//  Bar Series Implementation
// =========================================================

#[derive(Debug, Clone)]
pub struct BarSeries {
    pub name: String,
    pub values: Vec<f64>,
    pub y_key: String,

    // Appearance
    pub color: Color,
    pub bar_width: f64, // 0.0 - 1.0 relative to index step
}

impl BarSeries {
    pub fn new(name: impl Into<String>, color: Color) -> Self {
        Self {
            name: name.into(),
            values: Vec::new(),
            y_key: "Y".to_string(),
            color,
            bar_width: 0.6, // Default to 60% width
        }
    }

    pub fn axis(mut self, y_id: impl Into<String>) -> Self {
        self.y_key = y_id.into();
        self
    }

    pub fn bar_width(mut self, width: f64) -> Self {
        self.bar_width = width.max(0.1).min(1.0);
        self
    }

    pub fn push(mut self, value: f64) -> Self {
        self.values.push(value);
        self
    }

    pub fn extend(mut self, values: impl IntoIterator<Item = f64>) -> Self {
        self.values.extend(values);
        self
    }
}

impl Items<f64> for BarSeries {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, _theme: &Theme) {
        if self.values.is_empty() {
            return;
        }

        for (i, &val) in self.values.iter().enumerate() {
            let center = PlotPoint::new(i as f64, val / 2.0);

            plot.add_shape(
                Rectangle::new(
                    center,
                    Length::Plot(self.bar_width),
                    Length::Plot(val.abs()),
                )
                .fill(self.color),
            );
        }
    }
}

// =========================================================
//  Polymorphic Series Wrapper
// =========================================================

#[derive(Debug, Clone)]
pub enum Series {
    Line(LineSeries),
    Bar(BarSeries),
}

impl Series {
    pub fn y_key(&self) -> &str {
        match self {
            Series::Line(s) => &s.y_key,
            Series::Bar(s) => &s.y_key,
        }
    }

    pub fn values(&self) -> &[f64] {
        match self {
            Series::Line(s) => &s.values,
            Series::Bar(s) => &s.values,
        }
    }

    pub fn values_mut(&mut self) -> &mut Vec<f64> {
        match self {
            Series::Line(s) => &mut s.values,
            Series::Bar(s) => &mut s.values,
        }
    }
}

impl Items<f64> for Series {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        match self {
            Series::Line(s) => s.draw(plot, theme),
            Series::Bar(s) => s.draw(plot, theme),
        }
    }
}

// =========================================================
//  Combined Chart (The Coordinator)
// =========================================================

pub struct CombinedChart {
    state: State<AxisId, f64>,
    series: Vec<Series>,
    labels: Vec<String>,
    defined_axes: Vec<String>,
}

impl CombinedChart {
    pub const X: &'static str = "X";
    pub const Y: &'static str = "Y";

    pub fn new() -> Self {
        Self {
            state: State::new(),
            series: Vec::new(),
            labels: Vec::new(),
            defined_axes: Vec::new(),
        }
    }

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

    // --- Configuration ---

    pub fn with_axis(&mut self, id: impl Into<String>, axis: Axis<f64>) {
        let key = id.into();
        self.state.set_axis(key.clone(), axis);
        if !self.defined_axes.contains(&key) {
            self.defined_axes.push(key);
        }
        self.auto_scale();
    }

    pub fn push_series(&mut self, series: Series) {
        self.ensure_axes_exist(&series);
        self.series.push(series);
        self.auto_scale();
    }

    // --- Data Injection ---

    pub fn push(&mut self, label: impl Into<String>, value: f64) {
        let label = label.into();

        if self.series.is_empty() {
            let default = Series::Line(LineSeries::new("Series 1", Color::from_rgb(0.2, 0.4, 0.8)));
            self.ensure_axes_exist(&default);
            self.series.push(default);
        }

        // Logic split to satisfy borrow checker:
        // 1. Determine if we need to update labels based on the last series' current length
        let mut update_labels = false;
        if let Some(last) = self.series.last() {
            if last.values().len() >= self.labels.len() {
                update_labels = true;
            }
        }

        // 2. Perform label update if needed
        if update_labels {
            self.labels.push(label);
            self.update_x_axis_labels();
        }

        // 3. Perform value push
        if let Some(last) = self.series.last_mut() {
            last.values_mut().push(value);
        }

        self.auto_scale();
    }

    pub fn push_value(&mut self, value: f64) {
        self.push("", value);
    }

    pub fn clear(&mut self) {
        if let Some(first) = self.series.get_mut(0) {
            first.values_mut().clear();
        }
        self.labels.clear();
        self.auto_scale();
    }

    // --- Logic ---

    fn ensure_axes_exist(&mut self, series: &Series) {
        if !self.defined_axes.contains(&Self::X.to_string()) {
            self.state.set_axis(
                Self::X.to_string(),
                Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom),
            );
            self.defined_axes.push(Self::X.to_string());
            self.update_x_axis_labels();
        }

        let y_key = series.y_key();
        if !self.defined_axes.contains(&y_key.to_string()) {
            self.state.set_axis(
                y_key.to_string(),
                Axis::new(Linear::new(0.0, 1.0), axis::Position::Left),
            );
            self.defined_axes.push(y_key.to_string());
        }
    }

    fn update_x_axis_labels(&mut self) {
        let labels = self.labels.clone();
        let x_key = Self::X.to_string();

        let (min, max) = if let Some(a) = self.state.get_axis(&x_key) {
            let (min, max) = a.scale().domain();
            (*min, *max)
        } else {
            (0.0, 1.0)
        };

        // Use with_tick_renderer (Builder) instead of set_tick_renderer (Setter)
        // to correctly return the Axis struct.
        let axis = Axis::new(Linear::new(min, max), axis::Position::Bottom).with_tick_renderer(
            move |ctx| {
                let idx = ctx.tick.value.round();
                if (ctx.tick.value - idx).abs() > 0.001 {
                    return None;
                }
                let idx = idx as usize;
                if idx < labels.len() {
                    Some(TickLine::simple(labels[idx].clone()))
                } else {
                    None
                }
            },
        );

        self.state.set_axis(x_key, axis);
    }

    fn auto_scale(&mut self) {
        if self.series.is_empty() {
            return;
        }

        let max_len = self
            .series
            .iter()
            .map(|s| s.values().len())
            .max()
            .unwrap_or(0);
        let x_max = (max_len as f64 - 1.0).max(0.0);

        let x_key_str = Self::X.to_string();
        if let Some(x_axis) = self.state.get_axis_mut(&x_key_str) {
            x_axis.scale_mut().set_domain(0.0, x_max);
        }

        let mut y_bounds: HashMap<String, (f64, f64)> = HashMap::new();

        for s in &self.series {
            let values = s.values();
            if values.is_empty() {
                continue;
            }

            let mut min = f64::MAX;
            let mut max = f64::MIN;
            for &v in values {
                min = min.min(v);
                max = max.max(v);
            }

            let entry = y_bounds
                .entry(s.y_key().to_string())
                .or_insert((f64::MAX, f64::MIN));
            entry.0 = entry.0.min(min);
            entry.1 = entry.1.max(max);
        }

        for (axis_id, (min, max)) in y_bounds {
            if let Some(axis) = self.state.get_axis_mut(&axis_id) {
                let padding = if max > min { (max - min) * 0.05 } else { 1.0 };
                axis.scale_mut().set_domain(min, max + padding);
            }
        }
    }

    pub fn chart<Message>(&self) -> Chart<'_, AxisId, f64, Message> {
        let mut chart = Chart::new(&self.state);

        let mut sorted_series: Vec<&Series> = self.series.iter().collect();
        sorted_series.sort_by(|a, b| {
            let rank_a = match a {
                Series::Bar(_) => 0,
                Series::Line(_) => 1,
            };
            let rank_b = match b {
                Series::Bar(_) => 0,
                Series::Line(_) => 1,
            };
            rank_a.cmp(&rank_b)
        });

        for series in sorted_series {
            chart = chart.layer(series, Self::X.to_string(), series.y_key().to_string());
        }

        chart
    }
}
