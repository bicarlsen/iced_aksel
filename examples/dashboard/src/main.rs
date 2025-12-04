use std::{fmt::Display, time::Instant};

use aksel::scale::Linear;
use iced::{
    Color, Element, Subscription, Task, Theme,
    widget::{button, column, pick_list, row, text_input},
    window,
};

mod bar;
mod combined;
mod gauge;
mod line;

use bar::BarChart;
use gauge::Gauge;
use iced_aksel::{Axis, axis::Position};
use line::LineChart;
use rand::Rng;

use crate::combined::{BarSeries, LineSeries, Series};

fn main() -> iced::Result {
    ExampleApp::run()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SeriesType {
    Line,
    Bar,
}

impl Display for SeriesType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeriesType::Line => write!(f, "Line"),
            SeriesType::Bar => write!(f, "Bar"),
        }
    }
}

// --- Application State ---

struct ExampleApp {
    // Widgets
    series_type: SeriesType,
    input_value: f64,

    // Settings
    theme: iced::Theme,

    // Bar chart
    bar_chart: BarChart,
    gauge_chart: Gauge,
    line_chart: LineChart,

    // Customizable chart
    combined_state: combined::State,
}

#[derive(Debug, Clone)]
enum Message {
    // Animation
    AnimationTick(Instant),

    // Widget values
    SwitchTheme(iced::Theme),

    // Barchart
    AddBarData,
    ToggleOrientation,

    // Linechart
    // AddLineSeries,

    // Gauge
    UpdateGaugeValue(f64),

    // Customizablechart
    SyncChart,
    AddLineSeries,
    AddBarSeries,
    AddCustomData,
    ChartTypeChanged(SeriesType),
    InputValueChanged(String),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
                series_type: SeriesType::Bar,
                input_value: 0.0,

                theme: iced::Theme::Dark,

                bar_chart: BarChart::new(bar::Orientation::Vertical),
                gauge_chart: Gauge::new("Speed", 0., 100.)
                    .animated(0.5)
                    .value_pos(gauge::Placement::Center)
                    .title_pos(gauge::Placement::Custom(0.1, 0.9))
                    .zone(gauge::Zone::Success(65.))
                    .zone(gauge::Zone::Warning(75.))
                    .zone(gauge::Zone::Danger(100.))
                    .zone_opacity(0.7)
                    .format(|v| format!("{:.2}", v)),
                line_chart: LineChart::new(),
                combined_state: combined::State::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AnimationTick(now) => {
                self.gauge_chart.tick(now);
                Task::none()
            }
            Message::UpdateGaugeValue(value) => {
                let new_value = self.gauge_chart.get_value() + value;
                self.gauge_chart.set_value(new_value);
                Task::none()
            }
            Message::SwitchTheme(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::AddBarData => {
                let new_label = self.bar_chart.get_data().len();
                let new_value = rand::random_range(5.0..100.0);
                self.bar_chart
                    .add_data((format!["{}", new_label], new_value));

                self.line_chart.push_value_last_series(new_value);
                Task::none()
            }
            // Message::AddLineSeries => {
            // let series = LineSeries::new("Line series", generate_pastel_color());
            // self.line_chart.push_series(series);
            // Task::none()
            // }
            Message::ToggleOrientation => {
                self.bar_chart.toggle_orientation();
                Task::none()
            }
            Message::SyncChart => {
                // self.combined_state
                //     .sync(&self.series.data(), &vec!["1".to_string(), "2".to_string()]);
                Task::none()
            }
            Message::AddLineSeries => {
                let series = LineSeries::new("Line series", vec![], generate_pastel_color());
                self.combined_state
                    .add_series(Series::Line(series), "Y".to_string());
                Task::done(Message::SyncChart)
            }
            Message::AddBarSeries => {
                let series = BarSeries::new("Bar series", vec![], generate_pastel_color());
                self.combined_state
                    .add_series(Series::Bar(series), "Y".to_string());
                Task::done(Message::SyncChart)
            }
            Message::AddCustomData => {
                let rnd_num = rand::rng().random_range(0.0..=10.0);

                self.combined_state.add_data_to_last_series(rnd_num);
                Task::done(Message::SyncChart)
            }
            Message::ChartTypeChanged(series_type) => {
                self.series_type = series_type;
                Task::none()
            }
            Message::InputValueChanged(value) => {
                self.input_value = value.parse().unwrap_or(0.0);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- Theme toggle ---
        let theme_toggle = pick_list(iced::Theme::ALL, Some(&self.theme), |t| {
            Message::SwitchTheme(t)
        })
        .width(iced::Length::Fill);

        let chart = combined::CombinedChart::new(&self.combined_state).chart();

        let btn1 = button("Add LineSeries").on_press(Message::AddLineSeries);
        let btn2 = button("Add BarSeries").on_press(Message::AddBarSeries);
        let btn3 = button("Add Datapoint").on_press(Message::AddCustomData);

        let panel1 = row![btn1, btn2];

        column![theme_toggle, panel1, btn3, chart].into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::window::frames().map(Message::AnimationTick)
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .subscription(Self::subscription)
            .antialiasing(true)
            .run()
    }
}

/// Generate random pastel color
fn generate_pastel_color() -> Color {
    let blue = 128 + rand::random::<u8>() % 128;
    Color::from_rgb8(
        128 + rand::random::<u8>() % 128,
        128 + rand::random::<u8>() % 128,
        blue,
    )
}
