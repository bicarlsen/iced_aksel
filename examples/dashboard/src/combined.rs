use aksel::{PlotPoint, Scale, scale::Linear};
use iced::{
    Color, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Length, State as ChartState,
    axis::{self, GridLine, Orientation, TickLine},
    plot::{Items, Plot},
};
use rand::Rng;
use std::collections::{HashMap, HashSet};

// Import shapes
use iced_aksel::Stroke;
use iced_aksel::shape::{Label, Polygon, Polyline, Rectangle};

mod series;

pub use crate::combined::series::Series;
pub use crate::combined::series::bar::BarSeries;
pub use crate::combined::series::line::LineSeries;

type AxisId = String;

pub struct State {
    // Manages the axis for `Chart`
    chart_state: ChartState<AxisId, f64>,

    // Manages the data for the charting.
    data: Vec<(AxisId, Series)>,
}

impl State {
    pub const X_AXIS_ID: &'static str = "X";
    pub const Y_AXIS_ID: &'static str = "Y";

    pub fn new() -> Self {
        let mut chart_state = ChartState::new();

        // Initialize default axes
        chart_state.set_axis(
            Self::X_AXIS_ID.to_string(),
            Axis::new(Linear::new(0.0, 5.0), axis::Position::Bottom),
        );
        chart_state.set_axis(
            Self::Y_AXIS_ID.to_string(),
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Left),
        );

        Self {
            chart_state,
            data: vec![],
        }
    }

    pub fn get_all_series(&self) -> &[(AxisId, Series)] {
        &self.data
    }

    pub fn get_series_last(&self) -> Option<&Series> {
        self.data.last().map(|(_, series)| series)
    }

    // --- Series Management Pipeline ---
    pub fn add_series(&mut self, series: Series, y_axis_id: String) {
        self.ensure_axis_capacity(&series, &y_axis_id);
        self.data.push((y_axis_id, series));
        self.update_bounds();
    }

    pub fn add_data_to_last_series(&mut self, value: f64) {
        if self.data.is_empty() {
            return;
        }

        let idx = self.data.len() - 1;
        self.data[idx].1.push_value(value);

        self.update_bounds();
    }

    pub fn add_data_to_series(&mut self, idx: usize, value: f64) {
        if let Some((_, series)) = self.data.get_mut(idx) {
            series.push_value(value);
            self.update_bounds();
        }
    }

    // --- Synchronization Pipeline (Call in Update) ---

    pub fn sync(&mut self, labels: &[String]) {
        // 1. Cleanup
        self.prune_unused_axes();

        // 2. Update Scales
        self.update_bounds();

        // 3. Update Labels (Tick Renderer)
        self.update_x_axis_labels(labels);
    }

    /// Checks if a Y-axis exists for the new series. If not, creates it with a safe initial range.
    fn ensure_axis_capacity(&mut self, series: &Series, y_axis_id: &str) {
        if self.chart_state.get_axis(&y_axis_id.to_string()).is_some() {
            return;
        }

        let max_y = if series.values().is_empty() {
            100.0
        } else {
            series.highest_value() * 1.1 // 10% margin
        };

        self.chart_state.set_axis(
            y_axis_id.to_string(),
            Axis::new(Linear::new(0.0, max_y), axis::Position::Left),
        );
    }

    /// Removes axes from the chart state that are no longer referenced by any series.
    fn prune_unused_axes(&mut self) {
        let mut active_axes = Vec::new();
        active_axes.push(Self::X_AXIS_ID.to_string()); // Keep X

        for (axis_id, _) in &self.data {
            active_axes.push(axis_id.clone());
        }

        self.chart_state.retain_axes(&active_axes);
    }

    /// Triggers a recalculation of X and Y bounds based on current data.
    fn update_bounds(&mut self) {
        self.resize_x_axis();
        self.resize_y_axes();
    }

    fn resize_x_axis(&mut self) {
        let max_len = self
            .data
            .iter()
            .map(|(_, s)| s.values().len())
            .max()
            .unwrap_or(0);

        let has_bars = self.data.iter().any(|(_, s)| matches!(s, Series::Bar(_)));

        let (min, max) = if has_bars {
            // Bars need padding on sides (-0.6 to len+0.6)
            (-0.6, (max_len as f64 - 1.0).max(0.0) + 0.6)
        } else {
            // Lines fit tightly (0.0 to len-1.0)
            (0.0, (max_len as f64 - 1.0).max(0.0))
        };

        if let Some(axis) = self.chart_state.get_axis_mut(&Self::X_AXIS_ID.to_string()) {
            axis.scale_mut().set_domain(min, max);
        }
    }

    fn resize_y_axes(&mut self) {
        // Calculate max Y for every distinct axis
        let mut y_max_map: HashMap<String, f64> = HashMap::new();

        for (axis_id, series) in &self.data {
            let max = series.highest_value();
            let entry = y_max_map.entry(axis_id.clone()).or_insert(f64::MIN);
            *entry = entry.max(max);
        }

        // Apply bounds
        for (axis_id, max_val) in y_max_map {
            if let Some(axis) = self.chart_state.get_axis_mut(&axis_id) {
                let limit = if max_val == 0.0 { 10.0 } else { max_val * 1.1 };
                axis.scale_mut().set_domain(0.0, limit);
            }
        }
    }

    fn update_x_axis_labels(&mut self, labels: &[String]) {
        let x_id = Self::X_AXIS_ID.to_string();
        let labels = labels.to_vec();

        // Preserve current scale
        let (min, max) = if let Some(a) = self.chart_state.get_axis(&x_id) {
            let d = a.scale().domain();
            (*d.0, *d.1)
        } else {
            (0.0, 1.0)
        };

        let axis = Axis::new(Linear::new(min, max), axis::Position::Bottom).with_tick_renderer(
            move |ctx| {
                let val = ctx.tick.value;

                // Only show labels for integer indices
                if (val.round() - val).abs() > 0.001 {
                    return None;
                }

                let idx = val as isize;
                if idx < 0 || idx as usize >= labels.len() {
                    return None;
                }

                Some(TickLine::simple(labels[idx as usize].clone()))
            },
        );

        self.chart_state.set_axis(x_id, axis);
    }
}

pub struct CombinedChart<'a> {
    state: &'a State,
}

impl<'a> CombinedChart<'a> {
    pub fn new(state: &'a State) -> Self {
        Self { state }
    }

    pub fn chart<Message>(self) -> Chart<'a, AxisId, f64, Message> {
        let mut chart = Chart::new(&self.state.chart_state);

        let mut sorted_data: Vec<&(AxisId, Series)> = self.state.data.iter().collect();
        sorted_data.sort_by(|(_, a), (_, b)| {
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

        for (y_axis_id, series) in sorted_data {
            chart = chart.layer(series, State::X_AXIS_ID.to_string(), y_axis_id.clone());
        }

        chart
    }
}
