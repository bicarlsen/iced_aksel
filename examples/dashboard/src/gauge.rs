use aksel::{PlotPoint, Scale, scale::Linear};
use iced::{
    Color, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Length, State,
    axis::{self},
    plot::{Items, Plot},
};
use std::f32::consts::PI;

// Assuming Arc and Label are accessible via a common module (e.g., crate::shape).
use iced_aksel::shape::{Arc, Label};

type AxisId = &'static str;

/// A highly customizable Gauge widget.
pub struct Gauge {
    state: State<AxisId, f64>,

    // Data
    label: String,
    value: f64,
    min: f64,
    max: f64,
    unit: String,

    // Logic
    warning_threshold: f64,
    danger_threshold: f64,

    // Appearance / Configuration
    inner_radius_factor: f64, // 0.1 to 0.9 (thickness)
    start_angle: f32,         // Radians
    end_angle: f32,           // Radians
    decimals: usize,          // Number formatting
    tick_count: usize,        // Number of ticks to display
}

impl Gauge {
    const X_AXIS: &str = "X";
    const Y_AXIS: &str = "Y";

    /// Creates a new Gauge with default "Speedometer" styling (240-degree arc).
    pub fn new(
        label: impl Into<String>,
        value: f64,
        range: (f64, f64),
        unit: impl Into<String>,
    ) -> Self {
        let mut state = State::new();

        // 1. Setup Axes
        // We use a slightly wider domain to accommodate various shapes/angles safely.
        state.set_axis(
            Self::X_AXIS,
            Axis::new(Linear::new(-1.5, 1.5), axis::Position::Bottom).invisible(),
        );

        state.set_axis(
            Self::Y_AXIS,
            Axis::new(Linear::new(-1.5, 1.5), axis::Position::Left).invisible(),
        );

        // Default: 240 degrees, centered upwards
        let default_span_deg = 240.0;
        let half_span_rad = (default_span_deg / 2.0) * (PI / 180.0);

        // 1.5 * PI is "Top/Up" in this coordinate system
        let start_angle = (1.5 * PI) - half_span_rad;
        let end_angle = (1.5 * PI) + half_span_rad;

        Self {
            state,
            label: label.into(),
            value,
            min: range.0,
            max: range.1,
            unit: unit.into(),
            // Defaults
            warning_threshold: 0.75,
            danger_threshold: 0.90,
            inner_radius_factor: 0.75, // Standard thickness
            start_angle,
            end_angle,
            decimals: 0,
            tick_count: 0, // No ticks by default
        }
    }

    // --- Builder API ---

    /// Sets the warning and danger thresholds (0.0 - 1.0).
    pub fn with_thresholds(mut self, warning: f64, danger: f64) -> Self {
        self.warning_threshold = warning;
        self.danger_threshold = danger;
        self
    }

    /// Sets the thickness of the gauge bar.
    /// `factor`: 0.0 (thin) to 1.0 (full pie). Default is 0.75.
    pub fn thickness(mut self, factor: f64) -> Self {
        self.inner_radius_factor = factor.max(0.1).min(0.99);
        self
    }

    /// Sets the total angular span of the gauge in degrees.
    /// The gauge will always be centered upwards.
    /// e.g., 180 = Semi-circle, 240 = Speedometer, 360 = Full Circle.
    pub fn span(mut self, degrees: f64) -> Self {
        let half_span_rad = (degrees.to_radians() / 2.0) as f32;
        self.start_angle = (1.5 * PI) - half_span_rad;
        self.end_angle = (1.5 * PI) + half_span_rad;
        self
    }

    /// Sets the number of decimal places for the value label.
    pub fn precision(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    /// Adds tick marks to the inner edge of the gauge.
    pub fn ticks(mut self, count: usize) -> Self {
        self.tick_count = count;
        self
    }

    // --- Getters (State Inspection) ---

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn range(&self) -> (f64, f64) {
        (self.min, self.max)
    }

    pub fn thresholds(&self) -> (f64, f64) {
        (self.warning_threshold, self.danger_threshold)
    }

    pub fn get_thickness(&self) -> f64 {
        self.inner_radius_factor
    }

    /// Returns the total span in degrees.
    pub fn get_span(&self) -> f64 {
        (self.end_angle - self.start_angle).to_degrees() as f64
    }

    pub fn get_precision(&self) -> usize {
        self.decimals
    }

    pub fn get_ticks(&self) -> usize {
        self.tick_count
    }

    // --- Setters (Runtime Updates) ---

    /// Updates the current value to display.
    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }

    /// Updates the minimum and maximum range of the gauge.
    pub fn set_range(&mut self, min: f64, max: f64) {
        self.min = min;
        self.max = max;
    }

    /// Updates the warning and danger thresholds (0.0 - 1.0).
    pub fn set_thresholds(&mut self, warning: f64, danger: f64) {
        self.warning_threshold = warning;
        self.danger_threshold = danger;
    }

    /// Updates the thickness of the gauge bar (0.1 - 0.9).
    pub fn set_thickness(&mut self, factor: f64) {
        self.inner_radius_factor = factor.max(0.1).min(0.99);
    }

    /// Updates the total angular span in degrees (recalculates angles).
    pub fn set_span(&mut self, degrees: f64) {
        let half_span_rad = (degrees.to_radians() / 2.0) as f32;
        self.start_angle = (1.5 * PI) - half_span_rad;
        self.end_angle = (1.5 * PI) + half_span_rad;
    }

    /// Updates the number of decimal places shown in the center label.
    pub fn set_precision(&mut self, decimals: usize) {
        self.decimals = decimals;
    }

    /// Updates the number of tick marks.
    pub fn set_ticks(&mut self, count: usize) {
        self.tick_count = count;
    }

    // --- Iced Integration ---

    pub fn chart<Message>(&self) -> Chart<'_, AxisId, f64, Message> {
        Chart::new(&self.state).layer(self, Self::X_AXIS, Self::Y_AXIS)
    }

    // --- Internal Layout ---

    fn calculate_layout(&self) -> GaugeLayout {
        let center = PlotPoint::new(0.0, 0.0);

        // Geometry based on configuration
        let radius = Length::Plot(1.0);
        let inner_radius = Length::Plot(self.inner_radius_factor);

        // Zone ring sits just outside
        let zone_radius = Length::Plot(1.08);
        let zone_inner_radius = Length::Plot(1.02);

        let total_sweep = self.end_angle - self.start_angle;

        // Value Ratio
        let safe_denominator = if self.max == self.min {
            1.0
        } else {
            self.max - self.min
        };
        let value_ratio = ((self.value - self.min) / safe_denominator)
            .max(0.0)
            .min(1.0);

        // Angles
        let value_angle = self.start_angle + (value_ratio as f32 * total_sweep);
        let warn_angle = self.start_angle + (self.warning_threshold as f32 * total_sweep);
        let danger_angle = self.start_angle + (self.danger_threshold as f32 * total_sweep);

        GaugeLayout {
            center,
            radius,
            inner_radius,
            zone_radius,
            zone_inner_radius,
            start_angle: self.start_angle,
            end_angle: self.end_angle,
            total_sweep,
            value_angle,
            warn_angle,
            danger_angle,
            value_ratio,
        }
    }

    fn resolve_style(&self, theme: &Theme, value_ratio: f64) -> GaugeStyle {
        let palette = theme.palette();

        let color_safe = palette.success;
        let color_warn = palette.warning;
        let color_danger = palette.danger;

        let bar_color = if value_ratio >= self.danger_threshold {
            color_danger
        } else if value_ratio >= self.warning_threshold {
            color_warn
        } else {
            color_safe
        };

        GaugeStyle {
            track: Color {
                a: 0.1,
                ..palette.text
            },
            bar: bar_color,
            tick: Color {
                a: 0.5,
                ..palette.text
            },
            zone_safe: color_safe,
            zone_warning: color_warn,
            zone_danger: color_danger,
            value_text: bar_color,
            label_text: Color {
                a: 0.7,
                ..palette.text
            },
        }
    }
}

struct GaugeLayout {
    center: PlotPoint<f64>,
    radius: Length<f64>,
    inner_radius: Length<f64>,
    zone_radius: Length<f64>,
    zone_inner_radius: Length<f64>,
    start_angle: f32,
    end_angle: f32,
    total_sweep: f32,
    value_angle: f32,
    warn_angle: f32,
    danger_angle: f32,
    value_ratio: f64,
}

struct GaugeStyle {
    track: Color,
    bar: Color,
    tick: Color,
    zone_safe: Color,
    zone_warning: Color,
    zone_danger: Color,
    value_text: Color,
    label_text: Color,
}

// --- The Drawing Logic ---

impl Items<f64> for Gauge {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        let layout = self.calculate_layout();
        let style = self.resolve_style(theme, layout.value_ratio);

        // 1. Draw Outer Zone Ring
        plot.add_shape(
            Arc::new(
                layout.center,
                layout.zone_radius,
                layout.start_angle,
                layout.warn_angle,
            )
            .inner_radius(layout.zone_inner_radius)
            .fill(style.zone_safe),
        );
        plot.add_shape(
            Arc::new(
                layout.center,
                layout.zone_radius,
                layout.warn_angle,
                layout.danger_angle,
            )
            .inner_radius(layout.zone_inner_radius)
            .fill(style.zone_warning),
        );
        plot.add_shape(
            Arc::new(
                layout.center,
                layout.zone_radius,
                layout.danger_angle,
                layout.end_angle,
            )
            .inner_radius(layout.zone_inner_radius)
            .fill(style.zone_danger),
        );

        // 2. Draw Background Track
        plot.add_shape(
            Arc::new(
                layout.center,
                layout.radius,
                layout.start_angle,
                layout.end_angle,
            )
            .inner_radius(layout.inner_radius)
            .fill(style.track),
        );

        // 3. Draw Active Value Bar
        if layout.value_ratio > 0.001 {
            plot.add_shape(
                Arc::new(
                    layout.center,
                    layout.radius,
                    layout.start_angle,
                    layout.value_angle,
                )
                .inner_radius(layout.inner_radius)
                .fill(style.bar),
            );
        }

        // 4. Draw Ticks (If enabled)
        if self.tick_count > 1 {
            let step = layout.total_sweep / (self.tick_count as f32 - 1.0);

            // We draw ticks as very thin Arc slices at the inner radius
            let tick_len = Length::Plot(0.05); // Length of the tick mark
            let tick_pos = layout.inner_radius; // Position at inner edge

            // We need to resolve the inner radius value to calculate the "outer" part of the tick
            // For simplicity, we just subtract length from the inner radius (ticks go inward)
            let tick_inner = match layout.inner_radius {
                Length::Plot(v) => Length::Plot(v - 0.05),
                _ => Length::Plot(0.0), // Fallback
            };

            for i in 0..self.tick_count {
                let angle = layout.start_angle + (i as f32 * step);
                // Draw a 0.5 degree slice as a tick
                let half_deg = 0.5f32.to_radians();

                plot.add_shape(
                    Arc::new(layout.center, tick_pos, angle - half_deg, angle + half_deg)
                        .inner_radius(tick_inner)
                        .fill(style.tick),
                );
            }
        }

        // 5. Draw Center Value (Formatted)
        let value_text = format!("{:.p$} {}", self.value, self.unit, p = self.decimals);
        plot.add_shape(
            Label::new(value_text, PlotPoint::new(0.0, 0.2))
                .fill(style.value_text)
                .size(32.0)
                .align(Horizontal::Center, Vertical::Center),
        );

        // 6. Draw Title Label
        plot.add_shape(
            Label::new(&self.label, PlotPoint::new(0.0, -0.4))
                .fill(style.label_text)
                .size(16.0)
                .align(Horizontal::Center, Vertical::Center),
        );
    }
}
