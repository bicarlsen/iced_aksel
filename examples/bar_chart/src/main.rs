use std::f64::consts::PI;

use aksel::{Float, PlotPoint, PlotRect, scale::Linear};
use iced::{
    Color, Element, Task, Theme,
    widget::{button, column, pick_list, row, text_input},
};
use iced_aksel::{Axis, Chart, DragDelta, Shape, State, axis, plot::Items, shape::Polygon};

use crate::bar::BarChart;

mod bar;

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application State ---

struct ExampleApp {
    // Settings
    theme: iced::Theme,

    // Widget values
    new_label: String,
    new_value: f64,

    bar_chart: BarChart,
}

#[derive(Debug, Clone)]
enum Message {
    // Widget values
    SwitchTheme(iced::Theme),
    NewLabelChanged(String),
    NewValueChanged(String),

    AddData((String, f64)),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
                theme: iced::Theme::Dark,

                new_label: String::new(),
                new_value: 0.0,
                bar_chart: BarChart::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTheme(theme) => {
                self.theme = theme;
            }
            Message::NewLabelChanged(s) => {
                self.new_label = s;
            }
            Message::NewValueChanged(s) => {
                self.new_value = s.parse().unwrap_or(0.0);
            }
            Message::AddData(bar_data) => {
                self.bar_chart.data.push(bar_data.into());
                self.bar_chart.refresh();
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        // --- Theme toggle ---
        let theme_toggle = pick_list(iced::Theme::ALL, Some(&self.theme), |t| {
            Message::SwitchTheme(t)
        });

        let new_label_input = text_input("Jan", &self.new_label).on_input(Message::NewLabelChanged);
        let new_value_input =
            text_input("0", &self.new_value.to_string()).on_input(Message::NewValueChanged);
        let new_data_confirm =
            button("+").on_press(Message::AddData((self.new_label.clone(), self.new_value)));
        let bar_chart = self.bar_chart.view();

        let row_one = row![theme_toggle].spacing(16.).padding(16.);

        let row_two = row![new_label_input, new_value_input, new_data_confirm]
            .spacing(16.)
            .padding(16.);

        column![row_one, row_two, bar_chart].into()
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
