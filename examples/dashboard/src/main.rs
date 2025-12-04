use std::{fmt::Display, ops::Sub, time::Instant};

use aksel::scale::Linear;
use iced::{
    Color, Element, Subscription, Task, Theme,
    widget::{button, column, pick_list, row, text_input},
    window,
};

mod bar;
// mod combined;
mod gauge;
mod line;

use bar::BarChart;
use gauge::Gauge;
use iced_aksel::{Axis, axis::Position};
use line::LineChart;
use rand::{Rng, seq::IndexedRandom};

use crate::line::LineSeries;

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

    // Labels
    bar_chart_labels: Vec<String>,
    // Customizable chart
    // combined_state: combined::State,
}

#[derive(Debug, Clone)]
enum Message {
    // Animation
    AnimationTick(Instant),

    // General
    SwitchTheme(iced::Theme),

    // Data
    AddBarDataPoint,
    AddLineDataPoint,
    AddLineSeries,
    StackedLineChartToggle,
    BottomAlphaLineChartToggle,

    // Gauge
    UpdateGaugeValue(f64),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
                series_type: SeriesType::Bar,
                input_value: 0.0,

                theme: iced::Theme::Dark,

                bar_chart: BarChart::new(bar::Orientation::Vertical).animated(0.3),
                gauge_chart: Gauge::new("Speed", 0., 100.)
                    .animated(0.5)
                    .value_pos(gauge::Placement::Center)
                    .title_pos(gauge::Placement::Custom(0.1, 0.9))
                    .zone(gauge::Zone::Success(65.))
                    .zone(gauge::Zone::Warning(75.))
                    .zone(gauge::Zone::Danger(100.))
                    .zone_opacity(0.7)
                    .format(|v| format!("{:.2}", v)),
                line_chart: LineChart::new().legend(true).fill_alpha(0.25).animated(0.5),
                bar_chart_labels: vec![
                    "You".to_string(),
                    "Decide".to_string(),
                    "Your".to_string(),
                    "Own".to_string(),
                    "Labels".to_string(),
                ],
                // combined_state: combined::State::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Automation
            Message::AnimationTick(now) => {
                self.gauge_chart.tick(now);
                self.line_chart.tick(now);
                self.bar_chart.tick(now);
                Task::none()
            }

            // General
            Message::SwitchTheme(theme) => {
                self.theme = theme;
                Task::none()
            }
            // Data
            Message::AddBarDataPoint => {
                let len = self.bar_chart.get_data().len();
                let new_value = rand::random_range(0.0..=10.0);
                let label = format!("Label {}", len + 1);
                self.bar_chart.add_data((label, new_value));
                Task::none()
            }

            Message::AddLineDataPoint => {
                let new_value = {
                    if let Some(last) = self.line_chart.get_last() {
                        0.0f64.max(
                            last.values.last().unwrap_or(&10.0) + rand::random_range(-3.0..5.0),
                        )
                    } else {
                        10.0
                    }
                };
                self.line_chart.push_value_last_series(new_value);
                Task::none()
            }

            Message::AddLineSeries => {
                let new_series =
                    LineSeries::new("New Series", generate_random_pastel(&self.theme()));
                self.line_chart.push_series(new_series);
                Task::none()
            }

            Message::StackedLineChartToggle => {
                self.line_chart.toggle_stacked();
                Task::none()
            }

            Message::BottomAlphaLineChartToggle => {
                self.line_chart.toggle_fill();
                Task::none()
            }

            // GAUGE
            Message::UpdateGaugeValue(value) => {
                let new_value = self.gauge_chart.get_value() + value;
                self.gauge_chart.set_value(new_value);
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

        // let chart = combined::CombinedChart::new(&self.combined_state);

        // let btn1 = button("Add LineSeries").on_press(Message::AddLineSeries);
        // let btn2 = button("Add BarSeries").on_press(Message::AddBarSeries);
        // let btn3 = button("Add Datapoint").on_press(Message::AddCustomData);

        let bar_col = self.barchart_view();
        let gauge_col = self.gaugechart_view();
        let line_col = self.linechart_view();

        let top = row![bar_col, gauge_col];
        let mid = row![line_col];

        column![theme_toggle, top, mid].into()
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

    fn barchart_view(&self) -> Element<'_, Message> {
        let new_data_confirm_btn = button("+").on_press(Message::AddBarDataPoint);
        // let toggle_orientation_btn = button("Toggle orientation")
        //     .on_press(Message::ToggleOrientation)
        //     ;
        let bar_chart = self.bar_chart.chart();

        let panel = row![new_data_confirm_btn].spacing(16.);

        column![bar_chart, panel].into()
    }

    fn gaugechart_view(&self) -> Element<'_, Message> {
        let add_gauge_num = button("+").on_press(Message::UpdateGaugeValue(7.5));
        let sub_gauge_num = button("-").on_press(Message::UpdateGaugeValue(-7.5));

        let gauge_chart = self.gauge_chart.chart();

        let panel = row![add_gauge_num, sub_gauge_num].spacing(16.);

        column![gauge_chart, panel].into()
    }

    fn linechart_view(&self) -> Element<'_, Message> {
        let new_data_confirm_btn = button("Data +")
            .on_press(Message::AddLineDataPoint)
            .width(iced::Length::Fill);
        let new_series_confirm_btn = button("Series +")
            .on_press(Message::AddLineSeries)
            .width(iced::Length::Fill);
        let toggle_stacked_btn = button("Stack").on_press(Message::StackedLineChartToggle);
        let toggle_alpha_btn = button("Alpha").on_press(Message::BottomAlphaLineChartToggle);

        let line_chart = self.line_chart.chart();

        let panel = row![
            new_data_confirm_btn,
            new_series_confirm_btn,
            toggle_stacked_btn,
            toggle_alpha_btn
        ]
        .spacing(16.);

        column![line_chart, panel].into()
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

fn generate_random_pastel(theme: &Theme) -> Color {
    let palette = theme.palette();

    // 1. Create a pool of "vivid" candidates
    // We explicitly exclude 'background' and 'text' to ensure the result is colorful.
    let candidates = [
        palette.primary,
        palette.success,
        palette.danger,
        palette.warning, // Uncomment if your iced version supports warning
        palette.text,
    ];

    // 2. Pick a random color from the candidates
    let mut rng = rand::thread_rng();
    let selected_color = candidates.choose(&mut rng).unwrap_or(&palette.primary); // Fallback just in case

    // 3. Mix with white to make it pastel (0.6 is 60% white)
    mix_with_white(*selected_color, 0.6)
}

/// Helper: Mixes a color with white to tint it
fn mix_with_white(color: Color, factor: f32) -> Color {
    Color {
        r: color.r + (1.0 - color.r) * factor,
        g: color.g + (1.0 - color.g) * factor,
        b: color.b + (1.0 - color.b) * factor,
        a: color.a,
    }
}
