use aksel::{PlotPoint, Scale, scale::Linear};
use iced::{
    Color, Element, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Length, State as ChartState,
    axis::{self, GridLine, TickLine},
    plot::{Items, Plot},
};
use std::{collections::HashMap, time::Instant};

// Import shapes
use iced_aksel::Stroke;
use iced_aksel::shape::{Label, Polygon, Polyline, Rectangle};

type AxisId = String;

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub name: String,
    pub values: Vec<f64>,
    pub color: Color,
    pub width: f32,
    pub show_markers: bool,
    pub fill_color: Option<Color>,
}

/// The Enum allowing mixed types in a single list
#[derive(Debug, Clone)]
pub enum Series {
    Line(LineSeries),
    Bar(BarSeries),
}

// Holds all the values that needs persisting between frames
pub struct State {
    // Controls the axes
    chart_state: ChartState<AxisId, f64>,

    // Animation values
    last_tick: Option<Instant>,
}

pub struct CombinedChart<'a> {
    state: &'a State,
    // Find some way to combine a series to a axis
    series: ((AxidId, AxisId), &'a [Series]),

    width: iced::Length,
    height: iced::Length,
}

impl<'a> CombinedChart<'a> {
    pub fn new(
        state: &'a ChartState<AxisId, f64>,
        series: &'a [Series],
        labels: &'a [String],
    ) -> Self {
        todo!()
    }

    pub fn series(&mut self, series: &Series, x_id: AxisId, y_id: AxisId) {
        todo!()
    }

    // Uses builder pattern to properly wrap chart
    pub fn width(mut self) -> Self {
        todo!()
    }

    // --- The Main Render Function (Pure View) ---

    pub fn view<Message>(&self) -> Element<'_, Message> {
        let mut chart = Chart::new(&self.state.chart_state);

        for series in self.series {
            chart = chart.layer(series, x_axis_id, y_axis_id)
        }
    }
}

// =========================================================
//  4. RENDER IMPLEMENTATIONS (Items)
// =========================================================

impl Items<f64> for Series {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        match self {
            Series::Line(s) => s.draw(plot, theme),
            Series::Bar(s) => s.draw(plot, theme),
        }
    }
}

impl Items<f64> for LineSeries {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, _theme: &Theme) {}
}
