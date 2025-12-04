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
//  Line Series (Data Container)
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
            width: 2.5,
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

        let points: Vec<PlotPoint<f64>> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &v)| PlotPoint::new(i as f64, v))
            .collect();

        // 1. Draw Fill (Area)
        if let Some(fill_color) = self.fill_color {
            if let Some(first) = points.first() {
                if let Some(last) = points.last() {
                    // FIX: Use a virtual floor to ensure fill reaches the axis bottom.
                    // Using data min leaves a gap if the axis goes lower.
                    // -1.0e20 is effectively negative infinity for screen coords.
                    let min_y = -1.0e20;

                    let mut fill_points = points.clone();
                    fill_points.push(PlotPoint::new(last.x, min_y));
                    fill_points.push(PlotPoint::new(first.x, min_y));

                    plot.add_shape(Polygon::new(fill_points).fill(fill_color));
                }
            }
        }

        // 2. Draw Stroke
        plot.add_shape(Polyline {
            points: points.clone(),
            stroke: Stroke::new(self.color, Length::Screen(self.width)),
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 10.0,
        });

        // 3. Draw Markers
        if self.show_markers {
            for point in points {
                let marker_size = Length::Screen(self.width * 2.0 + 2.0);
                plot.add_shape(Rectangle::new(point, marker_size, marker_size).fill(self.color));
            }
        }
    }
}

// =========================================================
//  Line Chart (The Orchestrator & Renderer)
// =========================================================

pub struct LineChart {
    state: State<AxisId, f64>,
    series: Vec<LineSeries>,
    labels: Vec<String>,
    defined_axes: Vec<String>,

    // Config
    show_legend: bool,
    stacked: bool,

    // Fill Config
    fill_enabled: bool,
    fill_alpha: f32,        // Current alpha used for drawing
    target_fill_alpha: f32, // Saved preference for "On" state
}

impl LineChart {
    pub const X: &'static str = "X";
    pub const Y: &'static str = "Y";

    pub fn new() -> Self {
        Self {
            state: State::new(),
            series: Vec::new(),
            labels: Vec::new(),
            defined_axes: Vec::new(),
            show_legend: true,
            stacked: false,

            fill_enabled: false,
            fill_alpha: 0.0,
            target_fill_alpha: 0.2, // Default nice transparency
        }
    }

    pub fn with_default_axes(mut self) -> Self {
        self.with_axis(
            Self::X,
            Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom),
        );
        self.with_axis(Self::Y, y_axis(0.0, 1.0));
        self
    }

    // --- Configuration ---

    pub fn legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    /// Sets the target alpha for area fills.
    /// This updates the preference. If fill is currently enabled, it updates the view too.
    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        self.target_fill_alpha = alpha.max(0.0).min(1.0);

        if self.fill_enabled {
            self.fill_alpha = self.target_fill_alpha;
            self.update_series_fill();
        }
        self
    }

    /// Runtime setter for fill alpha preference.
    pub fn set_fill_alpha(&mut self, alpha: f32) {
        self.target_fill_alpha = alpha.max(0.0).min(1.0);
        if self.fill_enabled {
            self.fill_alpha = self.target_fill_alpha;
            self.update_series_fill();
        }
    }

    /// Toggles area filling on/off.
    pub fn toggle_fill(&mut self) {
        self.fill_enabled = !self.fill_enabled;

        self.fill_alpha = if self.fill_enabled {
            self.target_fill_alpha
        } else {
            0.0
        };

        self.update_series_fill();
    }

    /// Internal helper to propagate fill settings to all series
    fn update_series_fill(&mut self) {
        for s in &mut self.series {
            let mut color = s.color;
            color.a = self.fill_alpha;
            s.fill_color = if self.fill_alpha > 0.0 {
                Some(color)
            } else {
                None
            };
        }
    }

    pub fn stacked(mut self, stacked: bool) -> Self {
        self.stacked = stacked;
        self.auto_scale();
        self
    }

    pub fn set_stacked(&mut self, stacked: bool) {
        self.stacked = stacked;
        self.auto_scale();
    }

    pub fn toggle_stacked(&mut self) {
        self.stacked = !self.stacked;
        self.auto_scale();
    }

    pub fn with_axis(&mut self, id: impl Into<String>, axis: Axis<f64>) {
        let key = id.into();
        self.state.set_axis(key.clone(), axis);
        if !self.defined_axes.contains(&key) {
            self.defined_axes.push(key);
        }
        self.auto_scale();
    }

    pub fn push_series(&mut self, mut series: LineSeries) {
        // Apply global fill style to new series immediately
        if self.fill_alpha > 0.0 {
            let mut color = series.color;
            color.a = self.fill_alpha;
            series.fill_color = Some(color);
        }

        self.ensure_axes_exist(&series);
        self.series.push(series);
        self.auto_scale();
    }

    pub fn clear(&mut self) {
        self.series.clear();
        self.labels.clear();
        self.auto_scale();
    }

    pub fn get_last(&self) -> Option<&LineSeries> {
        self.series.last()
    }

    // --- Data Injection ---

    pub fn push(&mut self, label: impl Into<String>, value: f64) {
        let label = label.into();
        if self.series.is_empty() {
            let default_series = LineSeries::new("Data", Color::from_rgb(0.2, 0.4, 0.8));
            self.push_series(default_series);
        }

        let needs_label_update = if let Some(last) = self.series.last() {
            last.values.len() >= self.labels.len()
        } else {
            false
        };

        if needs_label_update {
            self.labels.push(label);
            self.update_x_axis_labels();
        }

        if let Some(last) = self.series.last_mut() {
            last.values.push(value);
        }
        self.auto_scale();
    }

    pub fn push_value(&mut self, value: f64) {
        self.push("", value);
    }

    pub fn push_to(&mut self, index: usize, label: impl Into<String>, value: f64) {
        let needs_label_update = if let Some(series) = self.series.get(index) {
            series.values.len() >= self.labels.len()
        } else {
            false
        };

        if needs_label_update {
            self.labels.push(label.into());
            self.update_x_axis_labels();
        }

        if let Some(series) = self.series.get_mut(index) {
            series.values.push(value);
            self.auto_scale();
        }
    }

    pub fn push_value_to(&mut self, index: usize, value: f64) {
        self.push_to(index, "", value);
    }

    pub fn push_value_last_series(&mut self, value: f64) {
        self.push_value(value);
    }

    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    // --- Internal Logic ---

    fn update_x_axis_labels(&mut self) {
        let labels = self.labels.clone();
        let x_key = Self::X.to_string();

        let (min, max) = if let Some(a) = self.state.get_axis(&x_key) {
            let d = a.scale().domain();
            (*d.0, *d.1)
        } else {
            (0.0, 1.0)
        };

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

    fn ensure_axes_exist(&mut self, series: &LineSeries) {
        let x_key = Self::X.to_string();
        if !self.defined_axes.contains(&x_key) {
            self.state.set_axis(
                x_key.clone(),
                Axis::new(Linear::new(0.0, 1.0), axis::Position::Bottom),
            );
            self.defined_axes.push(x_key);
            self.update_x_axis_labels();
        }

        if !self.defined_axes.contains(&series.y_key) {
            self.state.set_axis(series.y_key.clone(), y_axis(0., 1.));
            self.defined_axes.push(series.y_key.clone());
        }
    }

    fn auto_scale(&mut self) {
        if self.series.is_empty() {
            return;
        }

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

        let mut y_bounds: HashMap<String, (f64, f64)> = HashMap::new();

        if self.stacked {
            let mut stacked_sums: HashMap<String, Vec<f64>> = HashMap::new();
            for s in &self.series {
                let sums = stacked_sums.entry(s.y_key.clone()).or_insert_with(Vec::new);
                if s.values.len() > sums.len() {
                    sums.resize(s.values.len(), 0.0);
                }
                for (i, val) in s.values.iter().enumerate() {
                    sums[i] += val;
                }
            }
            for (key, sums) in stacked_sums {
                let max_h = sums.iter().fold(0.0f64, |a, &b| a.max(b));
                y_bounds.insert(key, (0.0, max_h));
            }
        } else {
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
        }

        for (axis_id, (min, max)) in y_bounds {
            if let Some(axis) = self.state.get_axis_mut(&axis_id) {
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

        if self.stacked {
            let first_y = self
                .series
                .first()
                .map(|s| s.y_key.clone())
                .unwrap_or(Self::Y.to_string());
            chart = chart.layer(self, Self::X.to_string(), first_y);
        } else {
            for series in &self.series {
                chart = chart.layer(series, Self::X.to_string(), series.y_key.clone());
            }

            if self.show_legend {
                chart = chart.layer(self, Self::X.to_string(), Self::Y.to_string());
            }
        }

        chart
    }
}

// Items Implementation for LineChart
// Handles Stacked Drawing AND Legend Drawing
impl Items<f64> for LineChart {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        // 1. Draw Stacked Lines (Only if mode is Stacked)
        if self.stacked {
            let mut baseline: Vec<f64> = Vec::new();

            for (idx, s) in self.series.iter().enumerate() {
                if s.values.len() < 2 {
                    continue;
                }

                if baseline.len() < s.values.len() {
                    baseline.resize(s.values.len(), 0.0);
                }

                // Calculate Top Points
                let top_points: Vec<PlotPoint<f64>> = s
                    .values
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| {
                        let total = baseline[i] + v;
                        PlotPoint::new(i as f64, total)
                    })
                    .collect();

                // Draw Fill
                // Note: Only use fill if alpha > 0
                if self.fill_alpha > 0.0 {
                    let mut fill_poly = top_points.clone();

                    if idx == 0 {
                        // FIX: First series in stack fills to Virtual Floor
                        let virtual_floor = -1.0e20;
                        if let (Some(first), Some(last)) = (top_points.first(), top_points.last()) {
                            fill_poly.push(PlotPoint::new(last.x, virtual_floor));
                            fill_poly.push(PlotPoint::new(first.x, virtual_floor));
                        }
                    } else {
                        // Connect back to baseline of previous series
                        for (i, &base_val) in baseline.iter().enumerate().take(s.values.len()).rev()
                        {
                            fill_poly.push(PlotPoint::new(i as f64, base_val));
                        }
                    }

                    // Generate color with alpha on the fly
                    let mut fill_color = s.color;
                    fill_color.a = self.fill_alpha;
                    plot.add_shape(Polygon::new(fill_poly).fill(fill_color));
                }

                // Draw Stroke
                plot.add_shape(Polyline {
                    points: top_points.clone(),
                    stroke: Stroke::new(s.color, Length::Screen(s.width)),
                    extend_start: false,
                    extend_end: false,
                    arrow_start: false,
                    arrow_end: false,
                    arrow_size: 10.0,
                });

                // Update Baseline
                for (i, &v) in s.values.iter().enumerate() {
                    baseline[i] += v;
                }
            }
        }

        // 2. Draw Legend
        if self.show_legend {
            let palette = theme.palette();

            if let (Some(x_axis), Some(y_axis)) = (
                self.state.get_axis(&Self::X.to_string()),
                self.state.get_axis(&Self::Y.to_string()),
            ) {
                let (x_min, x_max) = x_axis.scale().domain();
                let (y_min, y_max) = y_axis.scale().domain();

                let start_x = *x_min + (x_max - x_min) * 0.02;
                let start_y = *y_max - (y_max - y_min) * 0.05;
                let step_y = (y_max - y_min) * 0.06;

                for (i, series) in self.series.iter().enumerate() {
                    let y_pos = start_y - (i as f64 * step_y);

                    plot.add_shape(
                        Rectangle::new(
                            PlotPoint::new(start_x, y_pos),
                            Length::Screen(10.0),
                            Length::Screen(10.0),
                        )
                        .fill(series.color),
                    );

                    let text_offset = (x_max - x_min) * 0.02;

                    plot.add_shape(
                        Label::new(&series.name, PlotPoint::new(start_x + text_offset, y_pos))
                            .fill(palette.text)
                            .size(12.0)
                            .align(Horizontal::Left, Vertical::Center),
                    );
                }
            }
        }
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
