use iced_aksel::plot::Items;

use crate::combined::series::{bar::BarSeries, line::LineSeries};

pub mod bar;
pub mod line;

#[derive(Debug, Clone)]
pub enum Series {
    Line(LineSeries),
    Bar(BarSeries),
}
impl Series {
    pub fn highest_value(&self) -> f64 {
        match self {
            Series::Line(s) => s.highest_value(),
            Series::Bar(s) => s.highest_value(),
        }
    }

    pub fn lowest_value(&self) -> f64 {
        match self {
            Series::Line(s) => s.lowest_value(),
            Series::Bar(s) => s.lowest_value(),
        }
    }

    pub fn values(&self) -> &[f64] {
        match self {
            Series::Line(s) => &s.values,
            Series::Bar(s) => &s.values,
        }
    }

    pub fn push_value(&mut self, value: f64) {
        match self {
            Series::Line(s) => s.values.push(value),
            Series::Bar(s) => s.values.push(value),
        }
    }

    pub fn draw(
        &self,
        plot: &mut iced_aksel::plot::Plot<f64, iced::Renderer>,
        theme: &iced::Theme,
    ) {
        match self {
            Series::Line(s) => s.draw(plot, theme),
            Series::Bar(s) => s.draw(plot, theme),
        }
    }
}

impl iced_aksel::plot::Items<f64> for Series {
    fn draw(&self, plot: &mut iced_aksel::plot::Plot<f64, iced::Renderer>, theme: &iced::Theme) {
        match self {
            Series::Line(s) => s.draw(plot, theme),
            Series::Bar(s) => s.draw(plot, theme),
        }
    }
}
