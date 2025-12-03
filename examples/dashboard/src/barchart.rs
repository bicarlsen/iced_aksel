use aksel::{PlotPoint, Scale, Tick, TickIter, scale::Linear};
use iced::{
    Element, Theme,
    widget::{pick_list, text_input},
};
use iced_aksel::{
    Axis, Chart, Length, State,
    axis::{self, TickLine},
    plot::{Items, Plot},
    shape::Rectangle,
};

type AxisId = &'static str;

#[derive(Debug, Clone)]
pub struct BarData {
    pub label: String,
    pub value: f64,
}

impl From<(String, f64)> for BarData {
    fn from(data: (String, f64)) -> Self {
        Self {
            label: data.0,
            value: data.1,
        }
    }
}

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct BarChart {
    // State is owned here, making this a self-contained widget
    state: State<AxisId, f64>,
    data: Vec<BarData>,
    bar_width: f64,
    orientation: Orientation,
}

impl BarChart {
    const BAR_AXIS: &str = "BAR_AXIS";
    const VALUE_AXIS: &str = "VALUE_AXIS";

    pub fn new(orientation: Orientation) -> Self {
        let mut state = State::new();

        Self::setup_scales(&mut state, &orientation);

        let mut chart = Self {
            state,
            data: Vec::new(),
            bar_width: 0.8, // Default nice width (relative to x-step of 1.0)
            orientation,
        };

        // Refresh the chart before returning
        chart.refresh();

        chart
    }

    pub fn get_data(&self) -> &Vec<BarData> {
        &self.data
    }

    pub fn add_data<T: Into<BarData>>(&mut self, bar_data: T) {
        self.data.push(bar_data.into());
        self.refresh();
    }

    pub fn toggle_orientation(&mut self) {
        self.orientation = match self.orientation {
            Orientation::Horizontal => Orientation::Vertical,
            Orientation::Vertical => Orientation::Horizontal,
        };

        Self::setup_scales(&mut self.state, &self.orientation);

        self.refresh();
    }

    /// Replaces the current data and re-calculates the axis scales
    pub fn refresh(&mut self) {
        self.auto_scale();
        self.update_labels();
    }

    /// Returns the Chart widget.
    ///
    /// We return 'Chart' instead of 'Element' so the user can chain
    /// .on_drag(), .width(), or .height() before calling .into().
    pub fn chart<Message>(&self) -> Chart<'_, AxisId, f64, Message> {
        let (x_axis_id, y_axis_id) = match self.orientation {
            Orientation::Horizontal => (Self::VALUE_AXIS, Self::BAR_AXIS),
            Orientation::Vertical => (Self::BAR_AXIS, Self::VALUE_AXIS),
        };

        Chart::new(&self.state).layer(self, x_axis_id, y_axis_id)
    }

    /// Internal helper to fit the axes to the data
    fn auto_scale(&mut self) {
        let count = self.data.len() as f64;

        // Update Bar Axis: 0 to count (e.g., 5 items = 0..5)
        // We range from -0.5 to count-0.5 so the first (index 0) and last bars are fully visible
        if let Some(bar_axis) = self.state.get_axis_mut(&Self::BAR_AXIS) {
            bar_axis.scale_mut().set_domain(0.0, count + 1.0);
        }

        // Update Value Axis: 0 to max_value * 1.1 (10% headroom)
        if let Some(value_axis) = self.state.get_axis_mut(&Self::VALUE_AXIS) {
            // Find max Y value (or default to 10.0 if empty to avoid 0 range)
            let max_value = self
                .data
                .iter()
                .map(|d| d.value)
                .fold(0.0, f64::max)
                .max(10.0)
                * 1.05; // Add 5% padding

            value_axis.scale_mut().set_domain(0.0, max_value);
        }
    }

    fn setup_scales(state: &mut State<AxisId, f64>, orientation: &Orientation) {
        let bar_scale = Linear::new_with_tick_generator(0.0, 1.0, |scale| {
            let (&start, &end) = scale.domain();
            TickIter::new((start as i64..end as i64).map(|n| Tick {
                value: n as f64,
                level: 0,
            }))
        });
        let value_scale = Linear::new(0.0, 1.0);

        let (bar_pos, value_pos) = match orientation {
            Orientation::Horizontal => (axis::Position::Left, axis::Position::Bottom),
            Orientation::Vertical => (axis::Position::Bottom, axis::Position::Left),
        };

        state.set_axis(
            Self::VALUE_AXIS,
            Axis::new(value_scale, value_pos).with_tick_renderer(|tlc| match tlc.tick.level {
                0 => Some(TickLine::simple(format!("{:.2}", tlc.tick.value))),
                _ => None,
            }),
        );
        state.set_axis(Self::BAR_AXIS, Axis::new(bar_scale, bar_pos).without_grid());
    }

    fn update_labels(&mut self) {
        let labels: Vec<_> = self.data.iter().map(|data| data.label.clone()).collect();
        self.state
            .get_axis_mut(&Self::BAR_AXIS)
            .unwrap()
            .set_tick_renderer(move |ctx| {
                let idx = ctx.tick.value;

                // Don't draw label on the axis bound
                if idx <= 0. {
                    return None;
                }

                // Fetch and draw all other labels
                if let Some(label) = labels.get((idx - 1.0) as usize) {
                    return Some(TickLine::simple(label.clone()));
                }

                None
            });
    }
}

// --- The Drawing Logic ---

impl Items<f64> for BarChart {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        let palette = theme.palette();

        // Use the theme's primary color for the bars
        let bar_color = palette.primary;

        for (i, item) in self.data.iter().enumerate() {
            let index = i as f64 + 1.0;
            let val = item.value;

            // GEOMETRY CALCULATION:
            // Rectangle::new takes the CENTER.
            // To make a bar sit on the baseline (y=0) and go up to 'val':
            // Center Y = val / 2.0
            // Height   = val
            let shape = match self.orientation {
                Orientation::Horizontal => {
                    let center = PlotPoint::new(val / 2.0, index);
                    Rectangle::new(center, Length::Plot(val), Length::Plot(self.bar_width))
                        .fill(bar_color)
                }
                Orientation::Vertical => {
                    let center = PlotPoint::new(index, val / 2.0);
                    Rectangle::new(center, Length::Plot(self.bar_width), Length::Plot(val))
                        .fill(bar_color)
                }
            };

            plot.add_shape(shape);
        }
    }
}
