use std::{fmt::Display, time::Instant};

use iced::{
    Alignment, Border, Color, Element, Length, Padding, Subscription, Task, Theme,
    widget::{Space, button, column, container, pick_list, row, text},
};

// Ensure your local modules are accessible
mod bar;
mod gauge;
mod line;

use bar::BarChart;
use gauge::Gauge;
use line::LineChart;
use rand::seq::IndexedRandom;

use crate::line::LineSeries;

pub fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application State ---

struct ExampleApp {
    theme: Theme,
    bar_chart: BarChart,
    gauge_chart: Gauge,
    line_chart: LineChart,
}

#[derive(Debug, Clone)]
enum Message {
    AnimationTick(Instant),
    SwitchTheme(Theme),
    // Data - Line
    AddLineDataPoint,
    AddLineSeries,
    ToggleStacked,
    ToggleFill,
    // Data - Bar
    AddBarDataPoint,
    ToggleOrientation,
    // Data - Gauge
    UpdateGaugeValue(f64),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::Dark,

                bar_chart: BarChart::new(bar::Orientation::Vertical).animated(0.3),

                gauge_chart: Gauge::new("Speed", 0., 100.)
                    .animated(0.5)
                    .value_pos(gauge::Placement::Center)
                    .title_pos(gauge::Placement::Custom(0.5, 0.90))
                    .zone(gauge::Zone::Success(65.))
                    .zone(gauge::Zone::Warning(75.))
                    .zone(gauge::Zone::Danger(100.))
                    .zone_opacity(0.7)
                    .format(|v| format!("{:.1}", v)),

                line_chart: LineChart::new().legend(true).fill_alpha(0.25).animated(0.5),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AnimationTick(now) => {
                self.gauge_chart.tick(now);
                self.line_chart.tick(now);
                self.bar_chart.tick(now);
                Task::none()
            }
            Message::SwitchTheme(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::AddBarDataPoint => {
                let len = self.bar_chart.get_data().len();
                let new_value = rand::random_range(0.0..=10.0);
                let label = format!("Label {}", len + 1);
                self.bar_chart.add_data((label, new_value));
                Task::none()
            }
            Message::AddLineDataPoint => {
                self.line_chart.push_value(rand::random_range(0.0..10.0));
                Task::none()
            }
            Message::AddLineSeries => {
                let new_series = LineSeries::new("New Series", generate_random_pastel(&self.theme));
                self.line_chart.push_series(new_series);
                Task::none()
            }
            Message::ToggleStacked => {
                self.line_chart.toggle_stacked();
                Task::none()
            }
            Message::ToggleFill => {
                self.line_chart.toggle_fill();
                Task::none()
            }
            Message::ToggleOrientation => {
                self.bar_chart.toggle_orientation();
                Task::none()
            }
            Message::UpdateGaugeValue(value) => {
                let new_value = self.gauge_chart.get_value() + value;
                self.gauge_chart.set_value(new_value);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- 1. Bar Chart Section ---
        let bar_controls = row![
            button("Add Point")
                .on_press(Message::AddBarDataPoint)
                .width(Length::Fill),
            button("Rotate")
                .on_press(Message::ToggleOrientation)
                .width(Length::Fill),
        ]
        .spacing(10);

        // NOTE: Controls are now outside the glassy_container
        let bar_section = column![
            glassy_container(self.bar_chart.chart().padding(Padding::new(20.)))
                .height(Length::Fill),
            Space::default().height(10.0),
            bar_controls
        ];

        // --- 2. Gauge Section ---
        let gauge_controls = row![
            button("- 10")
                .on_press(Message::UpdateGaugeValue(-10.0))
                .width(Length::Fill),
            button("+ 10")
                .on_press(Message::UpdateGaugeValue(10.0))
                .width(Length::Fill),
        ]
        .spacing(10);

        let gauge_section = column![
            glassy_container(
                container(self.gauge_chart.chart())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
            )
            .height(Length::Fill),
            Space::default().height(10.0),
            gauge_controls
        ];

        // --- 3. Line Chart Section ---
        let line_controls = row![
            button("Add Data")
                .on_press(Message::AddLineDataPoint)
                .width(Length::Fill),
            button("Add Series")
                .on_press(Message::AddLineSeries)
                .width(Length::Fill),
            button("Stack")
                .on_press(Message::ToggleStacked)
                .width(Length::Fill),
            button("Alpha")
                .on_press(Message::ToggleFill)
                .width(Length::Fill),
        ]
        .spacing(10);

        let line_section = column![
            glassy_container(self.line_chart.chart().padding(Padding::new(20.)))
                .height(Length::Fill),
            Space::default().height(10.0),
            line_controls
        ];

        // --- Layout Assembly ---

        // Top Row: 2/3 for Bar, 1/3 for Gauge
        let top_row = row![
            bar_section.width(Length::FillPortion(2)),
            gauge_section.width(Length::FillPortion(1)),
        ]
        .spacing(20)
        .height(Length::FillPortion(1));

        // Bottom Row: Full Width
        let bottom_row = line_section
            .width(Length::Fill)
            .height(Length::FillPortion(1));

        // Theme Switcher
        let theme_picker =
            pick_list(Theme::ALL, Some(&self.theme), Message::SwitchTheme).width(Length::Fill);

        // Final Combine
        column![theme_picker, top_row, bottom_row]
            .spacing(20)
            .padding(20)
            .into()
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(|app: &ExampleApp| app.theme.clone())
            .subscription(Self::subscription)
            .antialiasing(true)
            .run()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::window::frames().map(Message::AnimationTick)
    }
}

/// The specific styling you requested:
/// Background = Text Color @ 2.5% Alpha
/// Border = Text Color @ 10% Alpha
fn glassy_container<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    container(content).style(|t| {
        let palette = t.palette();

        let mut bg = palette.text;
        bg.a = 0.025;

        let mut border_color = palette.text;
        border_color.a = 0.1;

        container::Style::default().background(bg).border(Border {
            color: border_color,
            width: 1.0,
            radius: 8.0.into(),
        })
    })
}

// --- Helpers ---

fn generate_random_pastel(theme: &Theme) -> Color {
    let palette = theme.palette();
    let candidates = [
        palette.primary,
        palette.success,
        palette.danger,
        palette.text,
    ];

    let mut rng = rand::rng();
    let selected = candidates.choose(&mut rng).unwrap_or(&palette.primary);
    mix_with_white(*selected, 0.6)
}

fn mix_with_white(color: Color, factor: f32) -> Color {
    Color {
        r: color.r + (1.0 - color.r) * factor,
        g: color.g + (1.0 - color.g) * factor,
        b: color.b + (1.0 - color.b) * factor,
        a: color.a,
    }
}
