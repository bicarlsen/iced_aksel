use aksel::{PlotPoint, Scale, scale::Linear};
use iced::{
    Color, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Length, State as ChartState,
    axis::{self},
    plot::{Items, Plot},
};
use std::f32::consts::PI;
use std::time::Instant;

use iced_aksel::shape::{Arc, Label};

type AxisId = &'static str;

// --- Helper Types ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zone {
    Primary(f64),
    Success(f64),
    Warning(f64),
    Danger(f64),
    Custom(f64, Color),
}

impl Zone {
    /// Helper to extract the threshold value regardless of the variant
    pub fn threshold(&self) -> f64 {
        match self {
            Zone::Primary(t) => *t,
            Zone::Success(t) => *t,
            Zone::Warning(t) => *t,
            Zone::Danger(t) => *t,
            Zone::Custom(t, _) => *t,
        }
    }

    /// Resolves the actual color using the current Theme palette
    pub fn resolve_color(&self, palette: &iced::theme::Palette) -> Color {
        match self {
            Zone::Primary(_) => palette.primary,
            Zone::Success(_) => palette.success,
            Zone::Warning(_) => palette.warning,
            Zone::Danger(_) => palette.danger,
            Zone::Custom(_, color) => *color,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Placement {
    Top,
    Bottom,
    Center,
    Hidden,
    Custom(f64, f64),
}

// --- The Gauge (Persistent Widget) ---

pub struct Gauge {
    chart_state: ChartState<AxisId, f64>,

    // Config
    label: String,
    unit: String,
    min: f64,
    max: f64,
    zones: Vec<Zone>,
    base_color: Option<Color>,
    decimals: usize,
    custom_formatter: Option<Box<dyn Fn(f64) -> String>>,
    title_placement: Placement,
    value_placement: Placement,
    inner_radius_factor: f64,
    start_angle: f32,
    end_angle: f32,
    tick_count: usize,

    // Physics
    value: f64,
    target_value: f64,
    animation_speed: Option<f64>,
    last_tick: Option<Instant>,
}

impl Gauge {
    const X_AXIS: &str = "X";
    const Y_AXIS: &str = "Y";

    pub fn new(label: impl Into<String>, min: f64, max: f64) -> Self {
        let mut chart_state = ChartState::new();
        chart_state.set_axis(
            Self::X_AXIS,
            Axis::new(Linear::new(-1.5, 1.5), axis::Position::Bottom).invisible(),
        );
        chart_state.set_axis(
            Self::Y_AXIS,
            Axis::new(Linear::new(-1.5, 1.5), axis::Position::Left).invisible(),
        );

        let default_span_deg = 240.0;
        let half_span_rad = (default_span_deg / 2.0) * (PI / 180.0);
        let start_angle = (1.5 * PI) - half_span_rad;
        let end_angle = (1.5 * PI) + half_span_rad;

        Self {
            chart_state,
            label: label.into(),
            unit: String::new(),
            min,
            max,
            zones: Vec::new(),
            base_color: None,
            decimals: 0,
            custom_formatter: None,
            title_placement: Placement::Bottom,
            value_placement: Placement::Center,
            inner_radius_factor: 0.75,
            start_angle,
            end_angle,
            tick_count: 0,
            value: min,
            target_value: min,
            animation_speed: None,
            last_tick: None,
        }
    }

    // =========================================================
    //  Runtime Logic (The "Must Haves" for Update Loop)
    // =========================================================

    pub fn set_value(&mut self, value: f64) {
        self.target_value = value;
        if self.animation_speed.is_none() {
            self.value = value;
        } else if self.last_tick.is_none() {
            self.last_tick = Some(Instant::now());
        }
    }

    pub fn tick(&mut self, now: Instant) {
        let Some(speed_normalized) = self.animation_speed else {
            return;
        };

        let dt = if let Some(last) = self.last_tick {
            (now - last).as_secs_f32()
        } else {
            0.0
        };
        self.last_tick = Some(now);

        let diff = self.value - self.target_value;
        if diff.abs() < f64::EPSILON {
            return;
        }

        let physics_speed = speed_normalized * 10.0;
        let decay = (-physics_speed * dt as f64).exp();
        self.value = self.target_value + diff * decay;

        let range = (self.max - self.min).abs().max(1.0);
        if (self.value - self.target_value).abs() < (range * 0.001) {
            self.value = self.target_value;
        }
    }

    // =========================================================
    //  Setters (Runtime Mutation)
    //  Use these if you need to change settings *after* init.
    // =========================================================

    pub fn set_range(&mut self, min: f64, max: f64) {
        self.min = min;
        self.max = max;
    }

    pub fn set_animation_speed(&mut self, speed: Option<f64>) {
        self.animation_speed = speed.map(|s| s.max(0.0).min(1.0));
    }

    pub fn set_unit(&mut self, unit: impl Into<String>) {
        self.unit = unit.into();
    }

    pub fn set_thickness(&mut self, factor: f64) {
        self.inner_radius_factor = factor.max(0.1).min(0.99);
    }

    pub fn set_span(&mut self, degrees: f64) {
        let half_span_rad = (degrees.to_radians() / 2.0) as f32;
        self.start_angle = (1.5 * PI) - half_span_rad;
        self.end_angle = (1.5 * PI) + half_span_rad;
    }

    pub fn set_ticks(&mut self, count: usize) {
        self.tick_count = count;
    }

    pub fn set_decimals(&mut self, decimals: usize) {
        self.decimals = decimals;
    }

    /// Clears existing zones and adds a new one.
    /// Useful if zones change dynamically based on data.
    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.push(zone);
        self.zones
            .sort_by(|a, b| a.threshold().partial_cmp(&b.threshold()).unwrap());
    }

    pub fn clear_zones(&mut self) {
        self.zones.clear();
    }

    pub fn set_title_pos(&mut self, placement: Placement) {
        self.title_placement = placement;
    }

    pub fn set_value_pos(&mut self, placement: Placement) {
        self.value_placement = placement;
    }

    // =========================================================
    //  Builder API (Initialization)
    //  Use these in `new()` for clean, one-line setup.
    // =========================================================

    pub fn animated(mut self, speed: f64) -> Self {
        self.set_animation_speed(Some(speed));
        self
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.set_unit(unit);
        self
    }

    pub fn thickness(mut self, factor: f64) -> Self {
        self.set_thickness(factor);
        self
    }

    pub fn span(mut self, degrees: f64) -> Self {
        self.set_span(degrees);
        self
    }

    pub fn ticks(mut self, count: usize) -> Self {
        self.set_ticks(count);
        self
    }

    pub fn decimals(mut self, count: usize) -> Self {
        self.set_decimals(count);
        self
    }

    /// Adds a Zone to the gauge.
    /// Example: `.zone(Zone::Success(50.0)).zone(Zone::Danger(100.0))`
    pub fn zone(mut self, zone: Zone) -> Self {
        self.add_zone(zone);
        self
    }

    pub fn base_color(mut self, color: Color) -> Self {
        self.base_color = Some(color);
        self
    }

    pub fn title_pos(mut self, placement: Placement) -> Self {
        self.set_title_pos(placement);
        self
    }

    pub fn value_pos(mut self, placement: Placement) -> Self {
        self.set_value_pos(placement);
        self
    }

    pub fn format<F>(mut self, formatter: F) -> Self
    where
        F: Fn(f64) -> String + 'static,
    {
        self.custom_formatter = Some(Box::new(formatter));
        self
    }

    // =========================================================
    //  View & Output
    // =========================================================

    pub fn chart<Message>(&self) -> Chart<'_, AxisId, f64, Message> {
        Chart::new(&self.chart_state).layer(self, Self::X_AXIS, Self::Y_AXIS)
    }

    // --- Getters ---
    pub fn get_value(&self) -> f64 {
        self.target_value
    }
    pub fn get_visual_value(&self) -> f64 {
        self.value
    }
    pub fn get_range(&self) -> (f64, f64) {
        (self.min, self.max)
    }
}

// --- Drawing Logic ---

impl Items<f64> for Gauge {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        let palette = theme.palette();

        // 1. Resolve Colors using the new Zone enum logic
        let active_color = if self.zones.is_empty() {
            self.base_color.unwrap_or(palette.primary)
        } else {
            self.zones
                .iter()
                .find(|z| self.value <= z.threshold())
                .map(|z| z.resolve_color(&palette))
                .unwrap_or_else(|| self.zones.last().unwrap().resolve_color(&palette))
        };

        let track_color = Color {
            a: 0.1,
            ..palette.text
        };

        // 2. Geometry
        let center = PlotPoint::new(0.0, 0.0);
        let radius = Length::Plot(1.0);
        let inner_radius = Length::Plot(self.inner_radius_factor);

        let total_sweep = self.end_angle - self.start_angle;
        let safe_denominator = if self.max == self.min {
            1.0
        } else {
            self.max - self.min
        };
        let value_ratio = ((self.value - self.min) / safe_denominator)
            .max(0.0)
            .min(1.0);
        let value_angle = self.start_angle + (value_ratio as f32 * total_sweep);

        // 3. Draw Zones
        if !self.zones.is_empty() {
            let zone_radius = Length::Plot(1.08);
            let zone_inner = Length::Plot(1.02);
            let mut current_angle = self.start_angle;

            for zone in &self.zones {
                let threshold = zone.threshold();
                let zone_color = zone.resolve_color(&palette);

                let zone_ratio = ((threshold - self.min) / safe_denominator)
                    .max(0.0)
                    .min(1.0);
                let zone_end_angle = self.start_angle + (zone_ratio as f32 * total_sweep);

                if zone_end_angle > current_angle {
                    let pastel = Color {
                        a: 0.5,
                        ..zone_color
                    };
                    plot.add_shape(
                        Arc::new(center, zone_radius, current_angle, zone_end_angle)
                            .inner_radius(zone_inner)
                            .fill(pastel),
                    );
                    current_angle = zone_end_angle;
                }
            }
        }

        // 4. Main Track
        plot.add_shape(
            Arc::new(center, radius, self.start_angle, self.end_angle)
                .inner_radius(inner_radius)
                .fill(track_color),
        );

        // 5. Active Bar
        if value_ratio > 0.001 {
            plot.add_shape(
                Arc::new(center, radius, self.start_angle, value_angle)
                    .inner_radius(inner_radius)
                    .fill(active_color),
            );
        }

        // 6. Ticks
        if self.tick_count > 1 {
            let step = total_sweep / (self.tick_count as f32 - 1.0);
            let tick_len = 0.05;
            let tick_pos = inner_radius;
            let tick_inner = match inner_radius {
                Length::Plot(v) => Length::Plot(v - tick_len),
                _ => Length::Plot(0.0),
            };
            let tick_color = Color {
                a: 0.5,
                ..palette.text
            };

            for i in 0..self.tick_count {
                let angle = self.start_angle + (i as f32 * step);
                let half_deg = 0.5f32.to_radians();
                plot.add_shape(
                    Arc::new(center, tick_pos, angle - half_deg, angle + half_deg)
                        .inner_radius(tick_inner)
                        .fill(tick_color),
                );
            }
        }

        // 7. Text
        let resolve_pos = |p: Placement| -> Option<(PlotPoint<f64>, Vertical)> {
            match p {
                Placement::Top => Some((PlotPoint::new(0.0, 0.4), Vertical::Bottom)),
                Placement::Bottom => Some((PlotPoint::new(0.0, -0.4), Vertical::Top)),
                Placement::Center => Some((PlotPoint::new(0.0, 0.2), Vertical::Center)),
                Placement::Custom(x, y) => Some((PlotPoint::new(x, y), Vertical::Center)),
                Placement::Hidden => None,
            }
        };

        if let Some((pos, vert)) = resolve_pos(self.value_placement) {
            let text = if let Some(fmt) = &self.custom_formatter {
                fmt(self.value)
            } else {
                format!("{:.p$}{}", self.value, self.unit, p = self.decimals)
            };
            plot.add_shape(
                Label::new(text, pos)
                    .fill(active_color)
                    .size(32.0)
                    .align(Horizontal::Center, vert),
            );
        }

        if let Some((pos, vert)) = resolve_pos(self.title_placement) {
            plot.add_shape(
                Label::new(&self.label, pos)
                    .fill(Color {
                        a: 0.7,
                        ..palette.text
                    })
                    .size(16.0)
                    .align(Horizontal::Center, vert),
            );
        }
    }
}
