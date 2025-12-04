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
use std::time::Instant;

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

    pub fn push(&mut self, value: f64) {
        self.values.push(value);
    }
}

// Implement Items for LineSeries (Standard Drawing)
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
                    let min_y = -1.0e20; // Virtual floor
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
    current_fill_alpha: f32,
    target_fill_alpha: f32, // User preference

    // Animation State
    animation_speed: Option<f64>,
    last_tick: Option<Instant>,
    opacity: f32, // 0.0 to 1.0 (Controls whole chart fade)
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
            current_fill_alpha: 0.0,
            target_fill_alpha: 0.2,

            animation_speed: None,
            last_tick: None,
            opacity: 1.0,
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

    pub fn animated(mut self, speed: f64) -> Self {
        self.animation_speed = Some(speed.max(0.0).min(1.0));
        self
    }

    pub fn legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        self.target_fill_alpha = alpha.max(0.0).min(1.0);
        // If fill is enabled, jump/start transitioning to new alpha
        if self.fill_enabled {
            if self.animation_speed.is_none() {
                self.current_fill_alpha = self.target_fill_alpha;
                self.update_series_fill();
            }
        }
        self
    }

    pub fn set_fill_alpha(&mut self, alpha: f32) {
        self.target_fill_alpha = alpha.max(0.0).min(1.0);
    }

    pub fn toggle_fill(&mut self) {
        self.fill_enabled = !self.fill_enabled;

        // If no animation, snap immediately
        if self.animation_speed.is_none() {
            self.current_fill_alpha = if self.fill_enabled {
                self.target_fill_alpha
            } else {
                0.0
            };
            self.update_series_fill();
        }
    }

    fn update_series_fill(&mut self) {
        for s in &mut self.series {
            let mut color = s.color;
            color.a = self.current_fill_alpha;
            s.fill_color = if self.current_fill_alpha > 0.0 {
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
        if self.stacked != stacked {
            self.stacked = stacked;

            // If animating, reset opacity to create a "Scene Cut" effect
            if self.animation_speed.is_some() {
                self.opacity = 0.0;
            }
            self.auto_scale();
        }
    }

    pub fn toggle_stacked(&mut self) {
        self.set_stacked(!self.stacked);
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
        if self.current_fill_alpha > 0.0 {
            let mut color = series.color;
            color.a = self.current_fill_alpha;
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

    // --- Physics ---

    pub fn tick(&mut self, now: Instant) {
        let Some(speed_normalized) = self.animation_speed else {
            return;
        };

        let dt = if let Some(last) = self.last_tick {
            (now - last).as_secs_f32() as f64
        } else {
            0.0
        };
        self.last_tick = Some(now);

        let physics_speed = speed_normalized * 10.0;
        let decay = (-physics_speed * dt).exp();

        // 1. Animate Opacity (Scene Transition)
        if self.opacity < 1.0 {
            let fade_speed = 3.0; // Speed of "fade in"
            self.opacity += (dt * fade_speed) as f32;
            if self.opacity > 1.0 {
                self.opacity = 1.0;
            }
        }

        // 2. Animate Fill Alpha
        let target_alpha = if self.fill_enabled {
            self.target_fill_alpha
        } else {
            0.0
        };
        let diff_alpha = target_alpha - self.current_fill_alpha;

        if diff_alpha.abs() > 1e-5 {
            // Use linear interpolation for alpha to prevent "never reaching 0" issues visually
            // or use the exponential decay for smoothness. Linear is often cleaner for fades.
            self.current_fill_alpha = target_alpha + diff_alpha * (decay as f32);
            self.update_series_fill();
        } else if self.current_fill_alpha != target_alpha {
            self.current_fill_alpha = target_alpha;
            self.update_series_fill();
        }
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
            last.push(value);
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
            // Stacked Scaling: Sum of values
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
            // Standard Scaling: Max of individual values
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

                // If stacked, ensure min is 0
                let final_min = if self.stacked { 0.0 } else { min };
                axis.scale_mut().set_domain(final_min, max + padding);
            }
        }
    }

    // =========================================================
    //  View
    // =========================================================

    pub fn chart<Message>(&self) -> Chart<'_, AxisId, f64, Message> {
        let mut chart = Chart::new(&self.state);

        // We use the chart itself as the renderer to handle the scene opacity
        // and stacking logic consistently.
        let first_y = self
            .series
            .first()
            .map(|s| s.y_key.clone())
            .unwrap_or(Self::Y.to_string());
        chart = chart.layer(self, Self::X.to_string(), first_y);

        chart
    }
}

// Unified Renderer
impl Items<f64> for LineChart {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        // --- Apply Scene Opacity ---
        let alpha_mod = self.opacity;

        // 1. Determine Chart Floor (Visually)
        let chart_floor = if let Some(axis) = self.state.get_axis(&Self::Y.to_string()) {
            *axis.scale().domain().0
        } else {
            0.0
        };

        // 2. Draw Series
        let mut baseline: Vec<f64> = Vec::new();

        for s in &self.series {
            if s.values.len() < 2 {
                continue;
            }

            if baseline.len() < s.values.len() {
                baseline.resize(s.values.len(), 0.0);
            }

            // Calculate Visual Points
            let points: Vec<PlotPoint<f64>> = s
                .values
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let effective_base = if self.stacked { baseline[i] } else { 0.0 };
                    let total = effective_base + v;
                    PlotPoint::new(i as f64, total)
                })
                .collect();

            // Draw Fill
            if let Some(fill_color) = s.fill_color {
                let mut fill_poly = points.clone();

                for (i, _) in s.values.iter().enumerate().take(s.values.len()).rev() {
                    let base_val = if self.stacked { baseline[i] } else { 0.0 };
                    let render_base = if self.stacked { base_val } else { -1.0e20 };

                    fill_poly.push(PlotPoint::new(i as f64, render_base));
                }

                let mut color = fill_color;
                color.a *= alpha_mod; // Fade in/out
                plot.add_shape(Polygon::new(fill_poly).fill(color));
            }

            // Draw Stroke
            let mut stroke_color = s.color;
            stroke_color.a *= alpha_mod; // Fade in/out

            plot.add_shape(Polyline {
                points: points.clone(),
                stroke: Stroke::new(stroke_color, Length::Screen(s.width)),
                extend_start: false,
                extend_end: false,
                arrow_start: false,
                arrow_end: false,
                arrow_size: 10.0,
            });

            // Draw Markers
            if s.show_markers {
                for point in &points {
                    let marker_size = Length::Screen(s.width * 2.0 + 2.0);
                    plot.add_shape(
                        Rectangle::new(*point, marker_size, marker_size).fill(stroke_color),
                    );
                }
            }

            // Accumulate
            for (i, &v) in s.values.iter().enumerate() {
                baseline[i] += v;
            }
        }

        // 3. Draw Legend
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

                    let mut legend_color = series.color;
                    legend_color.a *= alpha_mod;

                    plot.add_shape(
                        Rectangle::new(
                            PlotPoint::new(start_x, y_pos),
                            Length::Screen(10.0),
                            Length::Screen(10.0),
                        )
                        .fill(legend_color),
                    );

                    let text_offset = (x_max - x_min) * 0.02;

                    let mut text_color = palette.text;
                    text_color.a *= alpha_mod;

                    plot.add_shape(
                        Label::new(&series.name, PlotPoint::new(start_x + text_offset, y_pos))
                            .fill(text_color)
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
