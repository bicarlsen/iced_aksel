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

use crate::combined::Series;

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
    series: Vec<combined::Series>,
    customizable_chart_state: combined::State,
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
    AddLineSeries,

    // Gauge
    UpdateGaugeValue(f64),

    // Customizablechart
    SyncChart,
    AddCustomSeries,
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
                series: Vec::new(),
                customizable_chart_state: combined::State::new(),
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
            Message::AddLineSeries => {
                // let series = LineSeries::new("Line series", generate_pastel_color());
                // self.line_chart.push_series(series);
                Task::none()
            }
            Message::ToggleOrientation => {
                self.bar_chart.toggle_orientation();
                Task::none()
            }
            Message::SyncChart => {
                self.customizable_chart_state
                    .sync(&self.series, &vec!["1".to_string(), "2".to_string()]);
                Task::none()
            }
            Message::AddCustomSeries => {
                let series = match self.series_type {
                    SeriesType::Line => Series::Line(combined::LineSeries {
                        title: "Line series".to_string(),
                        values: vec![],
                        y_key: "y".to_string(),
                        color: generate_pastel_color(),
                        width: 2.0,
                        show_markers: true,
                        fill_color: None,
                    }),
                    SeriesType::Bar => Series::Bar(combined::BarSeries {
                        name: "Bar series".to_string(),
                        values: vec![],
                        y_key: "y".to_string(),
                        color: generate_pastel_color(),
                        bar_width: 0.7,
                    }),
                };
                self.series.push(series.into());
                Task::done(Message::SyncChart)
            }
            Message::AddCustomData => {
                if let Some(v) = self.series.last_mut() {
                    // v.push(self.input_value);
                }
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

        let bar_chart_box1 = column![self.barchart_view()].width(iced::Length::Fill);
        let bar_chart_box2 = column![self.gaugechart_view()].width(iced::Length::Fill);

        let row1 = row![bar_chart_box1, bar_chart_box2]
            .height(iced::Length::Fixed(360.))
            .spacing(16.)
            .padding(16.);

        let _row2 = row![self.linechart_view()]
            .height(iced::Length::Fixed(360.))
            .spacing(16.)
            .padding(16.);

        let row3 = row![self.combinedchart_view()]
            .height(iced::Length::Fixed(360.))
            .spacing(16.)
            .padding(16.);

        column![theme_toggle, row1, row3].into()
    }

    fn barchart_view(&self) -> Element<'_, Message> {
        let new_data_confirm_btn = button("+")
            .on_press(Message::AddBarData)
            .width(iced::Length::Fill);
        let toggle_orientation_btn = button("Toggle orientation")
            .on_press(Message::ToggleOrientation)
            .width(iced::Length::Fill);
        let bar_chart = self.bar_chart.chart();

        let panel = row![new_data_confirm_btn, toggle_orientation_btn].spacing(16.);

        column![bar_chart, panel].into()
    }

    fn gaugechart_view(&self) -> Element<'_, Message> {
        let add_gauge_num = button("+")
            .on_press(Message::UpdateGaugeValue(5.))
            .width(iced::Length::Fill);
        let sub_gauge_num = button("-")
            .on_press(Message::UpdateGaugeValue(-5.))
            .width(iced::Length::Fill);

        let gauge_chart = self.gauge_chart.chart();

        let panel = row![add_gauge_num, sub_gauge_num].spacing(16.);

        column![gauge_chart, panel].into()
    }

    fn linechart_view(&self) -> Element<'_, Message> {
        let new_data_confirm_btn = button("Data +")
            .on_press(Message::AddBarData)
            .width(iced::Length::Fill);
        let new_series_confirm_btn = button("Lineseries +")
            .on_press(Message::AddLineSeries)
            .width(iced::Length::Fill);

        let line_chart = self.line_chart.chart();

        let panel = row![new_data_confirm_btn, new_series_confirm_btn].spacing(16.);

        column![line_chart, panel].into()
    }

    fn combinedchart_view(&self) -> Element<'_, Message> {
        let type_picklist = pick_list(
            vec![SeriesType::Line, SeriesType::Bar],
            Some(self.series_type.clone()),
            Message::ChartTypeChanged,
        )
        .width(iced::Length::Fill);

        let new_series_confirm_btn = button("Series +")
            .on_press(Message::AddCustomSeries)
            .width(iced::Length::Fill);

        let new_value =
            text_input("xx", &self.input_value.to_string()).on_input(Message::InputValueChanged);

        let new_data_confirm_btn = button("Datapoint +")
            .on_press(Message::AddCustomData)
            .width(iced::Length::Fill);

        let combined_chart = CombinedChart::new(&self.customizable_chart_state, &self.series);

        let series_panel = row![type_picklist, new_series_confirm_btn].spacing(16.);
        let data_panel = row![new_value, new_data_confirm_btn].spacing(16.);

        column![series_panel, data_panel, combined_chart.view()].into()
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
