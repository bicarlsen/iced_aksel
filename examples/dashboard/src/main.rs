use std::time::Instant;

use aksel::scale::Linear;
use iced::{
    Color, Element, Subscription, Task, Theme,
    widget::{button, column, pick_list, row, text_input},
    window,
};

mod bar;
mod gauge;
mod line;

use bar::BarChart;
use gauge::Gauge;
use iced_aksel::{Axis, axis::Position};
use line::LineChart;

use crate::line::{DataPoint, LineSeries};

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application State ---

struct ExampleApp {
    // Settings
    theme: iced::Theme,

    // Bar chart
    bar_chart: BarChart,
    gauge_chart: Gauge,
    line_chart: LineChart,
}

#[derive(Debug, Clone)]
enum Message {
    // Animation
    AnimationTick(Instant),

    // Widget values
    SwitchTheme(iced::Theme),

    // Barchart
    AddData,
    ToggleOrientation,

    // Linechart
    AddSeries,

    // Gauge
    UpdateGaugeValue(f64),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
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
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AnimationTick(now) => {
                self.gauge_chart.tick(now);
            }
            Message::UpdateGaugeValue(value) => {
                let new_value = self.gauge_chart.get_value() + value;
                self.gauge_chart.set_value(new_value);
            }
            Message::SwitchTheme(theme) => {
                self.theme = theme;
            }
            Message::AddData => {
                let new_label = self.bar_chart.get_data().len();
                let new_value = rand::random_range(5.0..100.0);
                self.bar_chart
                    .add_data((format!["{}", new_label], new_value));

                self.line_chart.push_value_last_series(new_value);
            }
            Message::AddSeries => {
                let line_series = LineSeries::new("Line series", Color::WHITE, "X1", "Y1");
                self.line_chart.push_series(line_series);
            }
            Message::ToggleOrientation => {
                self.bar_chart.toggle_orientation();
            }
        }
        Task::none()
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

        let row2 = row![self.linechart_view()]
            .height(iced::Length::Fixed(360.))
            .spacing(16.)
            .padding(16.);

        column![theme_toggle, row1, row2].into()
    }

    fn barchart_view(&self) -> Element<'_, Message> {
        let new_data_confirm_btn = button("+")
            .on_press(Message::AddData)
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
            .on_press(Message::AddData)
            .width(iced::Length::Fill);
        let new_series_confirm_btn = button("Lineseries +")
            .on_press(Message::AddSeries)
            .width(iced::Length::Fill);

        let line_chart = self.line_chart.chart();

        let panel = row![new_data_confirm_btn, new_series_confirm_btn].spacing(16.);

        column![line_chart, panel].into()
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
