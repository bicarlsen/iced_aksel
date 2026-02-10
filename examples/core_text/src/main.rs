use iced::{
    Color, Element, Length, Theme,
    widget::{
        column, container, row,
        space::{horizontal, vertical},
        text,
    },
};
use iced_aksel::{Axis, Chart, Measure, PlotData, PlotPoint, State, scale::Linear, shape::Label};

fn main() -> iced::Result {
    iced::application(TextCanvas::new, TextCanvas::update, TextCanvas::view)
        .title("Text rendering")
        .theme(TextCanvas::theme)
        .antialiasing(true)
        .run()
}

struct TextCanvas {
    theme: Theme,
    state: State<&'static str, f64>,
    data: TextItems,
}

struct TextItems {
    labels: [Label<f64>; 3],
}

impl PlotData<f64> for TextItems {
    fn draw(&self, plot: &mut iced_aksel::Plot<f64>, theme: &iced::Theme) {
        let palette = theme.palette();
        for label in self.labels.clone() {
            plot.add_shape(label.fill(palette.text));
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ThemeChanged(Theme),
}

impl TextCanvas {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> Self {
        let state = State::new()
            .with_axis(
                Self::X,
                Axis::new(Linear::new(0.0, 10.0), iced_aksel::axis::Position::Bottom)
                    .invisible()
                    .without_grid(),
            )
            .with_axis(
                Self::Y,
                Axis::new(Linear::new(0.0, 10.0), iced_aksel::axis::Position::Left)
                    .invisible()
                    .without_grid(),
            );

        let data = TextItems {
            labels: [
                Label::new("Test æøå", PlotPoint::new(5.0, 9.0))
                    .size(Measure::Screen(12.0))
                    .quality(iced_aksel::Quality::High),
                Label::new("Test æøå", PlotPoint::new(5.0, 6.0))
                    .size(Measure::Screen(16.0))
                    .quality(iced_aksel::Quality::High),
                Label::new("Test æøå", PlotPoint::new(5.0, 3.0))
                    .size(Measure::Screen(24.0))
                    .quality(iced_aksel::Quality::High),
            ],
        };

        Self {
            theme: Theme::Dark,
            state,
            data,
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ThemeChanged(theme) => self.theme = theme,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let comparisons = column![
            text("Test æøå").size(12.0).height(Length::FillPortion(1)),
            text("Test æøå").size(16.0).height(Length::FillPortion(1)),
            text("Test æøå").size(24.0).height(Length::FillPortion(1))
        ];
        row![
            horizontal(),
            Chart::new(&self.state)
                .quality(1000.0)
                .plot_data(&self.data, Self::X, Self::Y),
            comparisons,
            horizontal(),
        ]
        .padding(100.0)
        .into()
    }
}
