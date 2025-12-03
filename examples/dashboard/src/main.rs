use iced::{
    Element, Task, Theme,
    widget::{button, column, pick_list, row, text_input},
};

mod barchart;

use barchart::BarChart;

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application State ---

struct ExampleApp {
    // Settings
    theme: iced::Theme,

    // Bar chart
    bar_chart: BarChart,
}

#[derive(Debug, Clone)]
enum Message {
    // Widget values
    SwitchTheme(iced::Theme),
    AddData,
    ToggleOrientation,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
                theme: iced::Theme::Dark,

                bar_chart: BarChart::new(barchart::Orientation::Vertical),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
        let bar_chart_box2 = column![self.barchart_view()].width(iced::Length::Fill);

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

        let row1 = row![new_data_confirm_btn, toggle_orientation_btn].spacing(16.);

        column![row1, bar_chart].into()
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
