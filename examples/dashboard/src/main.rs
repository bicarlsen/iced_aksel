use std::{fmt::Display, time::Instant};

use iced::{
    Alignment, Border, Color, Element, Length, Padding, Subscription, Task, Theme, font,
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

// --- Constants for "Curated" feel ---
const BRAND_COLORS: [Color; 5] = [
    Color::from_rgb(0.4, 0.6, 0.9), // Soft Blue
    Color::from_rgb(0.9, 0.4, 0.4), // Soft Red
    Color::from_rgb(0.4, 0.8, 0.5), // Soft Green
    Color::from_rgb(0.9, 0.7, 0.2), // Soft Yellow
    Color::from_rgb(0.7, 0.5, 0.9), // Soft Purple
];

struct ExampleApp {
    theme: Theme,
    bar_chart: BarChart,
    gauge_chart: Gauge,
    line_chart: LineChart,
    color_index: usize,
}

#[derive(Debug, Clone)]
enum Message {
    AnimationTick(Instant),
    SwitchTheme(Theme),
    AddLineDataPoint,
    AddLineSeries,
    ToggleStacked,
    AddBarDataPoint,
    ToggleOrientation,
    UpdateGaugeValue(f64),
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
                theme: Theme::Dark,
                color_index: 0,

                bar_chart: BarChart::new(bar::Orientation::Vertical).animated(0.3),

                // Fix: Pass empty string for title to "hide" it inside the chart
                // since we are rendering our own custom header outside.
                gauge_chart: Gauge::new("", 0., 100.)
                    .animated(0.5)
                    .value_pos(gauge::Placement::Center)
                    // Removed .title_pos(Placement::None) as it doesn't exist.
                    // Empty string above handles the "No Title" look.
                    .zone(gauge::Zone::Success(65.))
                    .zone(gauge::Zone::Warning(75.))
                    .zone(gauge::Zone::Danger(100.))
                    .zone_opacity(0.5)
                    .format(|v| format!("{:.0}", v)),

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
                let new_value = rand::random_range(1.0..=10.0);
                let label = format!("Q{}", len + 1);
                self.bar_chart.add_data((label, new_value));
                Task::none()
            }
            Message::AddLineDataPoint => {
                self.line_chart.push_value(rand::random_range(0.0..10.0));
                Task::none()
            }
            Message::AddLineSeries => {
                // Curated color cycling
                let color = BRAND_COLORS[self.color_index % BRAND_COLORS.len()];
                self.color_index += 1;

                let new_series = LineSeries::new("Metric", color);
                self.line_chart.push_series(new_series);
                Task::none()
            }
            Message::ToggleStacked => {
                self.line_chart.toggle_stacked();
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
        // --- 1. Top Section ---

        // A. Bar Chart Widget
        let bar_header_actions = row![
            action_button("⟳", Message::ToggleOrientation),
            action_button("+", Message::AddBarDataPoint),
        ]
        .spacing(5);

        // Fix: Use Padding::from(10.0) or 10.into()
        let bar_widget = widget_card(
            "Revenue Sources",
            self.bar_chart.chart().padding(Padding::from(10.0)),
            bar_header_actions,
            &self.theme,
        );

        // B. Gauge Widget
        let gauge_header_actions = row![
            action_button("-", Message::UpdateGaugeValue(-10.0)),
            action_button("+", Message::UpdateGaugeValue(10.0)),
        ]
        .spacing(5);

        let gauge_widget = widget_card(
            "System Load",
            container(self.gauge_chart.chart())
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill),
            gauge_header_actions,
            &self.theme,
        );

        // --- 2. Bottom Section ---

        // C. Line Chart Widget
        let line_header_actions = row![
            action_button("Data", Message::AddLineDataPoint),
            action_button("Series", Message::AddLineSeries),
            action_button("Stack", Message::ToggleStacked),
        ]
        .spacing(5);

        // Fix: Use Padding::from(10.0)
        let line_widget = widget_card(
            "Traffic History",
            self.line_chart.chart().padding(Padding::from(10.0)),
            line_header_actions,
            &self.theme,
        );

        // --- Layout ---
        let top_row = row![
            bar_widget.width(Length::FillPortion(2)),
            gauge_widget.width(Length::FillPortion(1)),
        ]
        .spacing(20)
        .height(Length::FillPortion(1));

        let bottom_row = line_widget
            .width(Length::Fill)
            .height(Length::FillPortion(1));

        let theme_picker =
            pick_list(Theme::ALL, Some(&self.theme), Message::SwitchTheme).width(Length::Fill);

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

// --- Professional UI Components ---

fn widget_card<'a>(
    title: &'a str,
    content: impl Into<Element<'a, Message>>,
    actions: impl Into<Element<'a, Message>>,
    theme: &Theme,
) -> container::Container<'a, Message> {
    // Fix: Space::new(Length::Fill, Length::Shrink) acts as a horizontal spacer
    let spacer = Space::new().width(Length::Fill).height(Length::Shrink);

    // Fix: Use .color() directly on text, not inside .style() closure
    let header = row![
        text(title)
            .font(font::Font::with_name("Inter"))
            .size(14)
            .color(theme.palette().text.scale_alpha(0.7)),
        spacer,
        actions.into()
    ]
    .align_y(Alignment::Center)
    .padding(Padding::new(0.0).bottom(10.0));

    let body = container(content).width(Length::Fill).height(Length::Fill);

    glassy_container(column![header, body])
}

fn glassy_container<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    container(content).padding(15).style(|t| {
        let palette = t.palette();

        let mut bg = palette.text;
        bg.a = 0.03;

        let mut border_color = palette.text;
        border_color.a = 0.08;

        container::Style::default().background(bg).border(Border {
            color: border_color,
            width: 1.0,
            radius: 12.0.into(),
        })
    })
}

// Fix: explicit lifetime 'a added to `label`
fn action_button<'a>(label: &'a str, msg: Message) -> button::Button<'a, Message> {
    button(text(label).size(12).align_x(Alignment::Center))
        .on_press(msg)
        .padding([4, 10])
        .style(button::secondary)
}
