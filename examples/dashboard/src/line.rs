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

// We import the specific shape and the Stroke definition
use iced_aksel::Stroke;
use iced_aksel::shape::Polyline;

type AxisId = &'static str;

#[derive(Debug, Clone, Copy)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
}

impl From<(f64, f64)> for DataPoint {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

/// A simple Line Chart that visualizes a single series of data.
pub struct LineChart {
    state: State<AxisId, f64>,

    // Data
    data: Vec<DataPoint>,

    // Appearance
    color: Option<Color>,
    thickness: f32,
}

impl LineChart {
    const X_AXIS: &str = "X";
    const Y_AXIS: &str = "Y";

    pub fn new(data: Vec<impl Into<DataPoint>>) -> Self {
        let mut state = State::new();

        // Convert input data
        let data: Vec<DataPoint> = data.into_iter().map(|d| d.into()).collect();

        // 1. Calculate Bounds for Auto-Scaling
        let (min_x, max_x, min_y, max_y) = if data.is_empty() {
            (0.0, 1.0, 0.0, 1.0)
        } else {
            let first = &data[0];
            data.iter().fold(
                (first.x, first.x, first.y, first.y),
                |(min_x, max_x, min_y, max_y), p| {
                    (
                        min_x.min(p.x),
                        max_x.max(p.x),
                        min_y.min(p.y),
                        max_y.max(p.y),
                    )
                },
            )
        };

        // Add 5% padding
        let padding_x = (max_x - min_x) * 0.05;
        let padding_y = (max_y - min_y) * 0.05;

        // 2. Setup Axes
        state.set_axis(
            Self::X_AXIS,
            Axis::new(
                Linear::new(min_x - padding_x, max_x + padding_x),
                axis::Position::Bottom,
            ),
        );

        state.set_axis(
            Self::Y_AXIS,
            Axis::new(
                Linear::new(min_y - padding_y, max_y + padding_y),
                axis::Position::Left,
            ),
        );

        Self {
            state,
            data,
            color: None,
            thickness: 2.0,
        }
    }

    // --- Builder API ---

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn thickness(mut self, width: f32) -> Self {
        self.thickness = width;
        self
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

        // 1. Prepare Points
        let points: Vec<PlotPoint<f64>> =
            self.data.iter().map(|p| PlotPoint::new(p.x, p.y)).collect();

        // 2. Create Stroke
        // Corrected signature based on your input: new(color, width)
        let stroke = Stroke::new(line_color, Length::Screen(self.thickness));

        // 3. Create Polyline
        let line = Polyline {
            points,
            stroke,
            // Standard line chart settings:
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 10.0,
        };

        plot.add_shape(line);
    }
}
