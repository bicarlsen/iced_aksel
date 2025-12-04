#[derive(Debug, Clone)]
pub struct BarSeries {
    pub name: String,
    pub values: Vec<f64>,
    pub color: iced::Color,
    pub bar_width: f64,
}

impl BarSeries {
    pub fn new(name: impl Into<String>, values: Vec<f64>, color: iced::Color) -> Self {
        Self {
            name: name.into(),
            values,
            color,
            bar_width: 0.6,
        }
    }

    // --- Configuration (Builder Pattern) ---

    pub fn bar_width(mut self, width: f64) -> Self {
        self.bar_width = width.max(0.1).min(1.0);
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

impl iced_aksel::plot::Items<f64> for BarSeries {
    fn draw(&self, plot: &mut iced_aksel::plot::Plot<f64, iced::Renderer>, _theme: &iced::Theme) {
        if self.values.is_empty() {
            return;
        }

        for (i, &val) in self.values.iter().enumerate() {
            let center = aksel::PlotPoint::new(i as f64, val / 2.0);
            plot.add_shape(
                iced_aksel::shape::Rectangle::new(
                    center,
                    iced_aksel::Length::Plot(self.bar_width),
                    iced_aksel::Length::Plot(val.abs()),
                )
                .fill(self.color),
            );
        }
    }
}
