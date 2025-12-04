use iced_aksel::{Length, Stroke};

#[derive(Debug, Clone)]
pub struct LineSeries {
    pub title: String,
    pub values: Vec<f64>,
    pub stroke: Stroke<f64>,

    pub visible: bool,
    pub show_markers: bool,
}

impl LineSeries {
    pub fn new(title: impl Into<String>, values: Vec<f64>, color: iced::Color) -> Self {
        Self {
            title: title.into(),
            values,
            stroke: Stroke::new(color, Length::Screen(2.)),
            visible: true,
            show_markers: false,
        }
    }
    pub fn markers(mut self, show: bool) -> Self {
        self.show_markers = show;
        self
    }

    // --- Data Manipulation (Mutable) ---

    pub fn push(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn extend(&mut self, values: impl IntoIterator<Item = f64>) {
        self.values.extend(values);
    }

    pub fn highest_value(&self) -> f64 {
        self.values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    }

    pub fn lowest_value(&self) -> f64 {
        self.values.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }
}

impl iced_aksel::plot::Items<f64> for LineSeries {
    fn draw(&self, plot: &mut iced_aksel::plot::Plot<f64, iced::Renderer>, _theme: &iced::Theme) {
        if self.values.len() < 2 {
            return;
        }

        let points: Vec<aksel::PlotPoint<f64>> = self
            .values
            .iter()
            .enumerate()
            .map(|(i, &v)| aksel::PlotPoint::new(i as f64, v))
            .collect();

        plot.add_shape(iced_aksel::shape::Polyline {
            points: points.clone(),
            stroke: iced_aksel::Stroke::new(self.stroke.fill, self.stroke.thickness),
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 10.0,
        });

        if self.show_markers {
            for point in points {
                let marker_size = self.stroke.thickness * 2.5;
                plot.add_shape(
                    iced_aksel::shape::Rectangle::new(point, marker_size, marker_size)
                        .fill(self.stroke.fill),
                );
            }
        }
    }
}
