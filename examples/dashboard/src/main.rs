use iced::{
    Element, Task, Theme,
    widget::{button, column, pick_list, row, text_input},
};

mod barchart;
mod gauge;

use barchart::BarChart;
use gauge::Gauge;

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
}

#[derive(Debug, Clone)]
enum Message {
    // Widget values
    SwitchTheme(iced::Theme),

    // Barchart
    AddData,
    ToggleOrientation,

    // Gauge
    UpdateGaugeValue(f64),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
                theme: iced::Theme::Dark,

                bar_chart: BarChart::new(barchart::Orientation::Vertical),
                gauge_chart: Gauge::new("Speed", 99., (0.0, 100.0), "ms"),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateGaugeValue(value) => {
                let new_value = self.gauge_chart.value() + value;
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

        column![theme_toggle, row1].into()
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
            .on_press(Message::UpdateGaugeValue(1.))
            .width(iced::Length::Fill);
        let sub_gauge_num = button("-")
            .on_press(Message::UpdateGaugeValue(-1.))
            .width(iced::Length::Fill);

        let gauge_chart = self.gauge_chart.chart();

        let panel = row![add_gauge_num, sub_gauge_num].spacing(16.);

        column![gauge_chart, panel].into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .run()
    }
}
