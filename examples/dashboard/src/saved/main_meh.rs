use iced::{
    Alignment, Color, Element, Length, Task, Theme,
    widget::{button, column, container, pick_list, row, scrollable, text},
};
mod combined;
use combined::{BarSeries, CombinedChart, LineSeries, Series, State};

pub fn main() -> iced::Result {
    iced::application(ExampleApp::init, ExampleApp::update, ExampleApp::view)
        .theme(ExampleApp::theme)
        .antialiasing(true)
        .run()
}

// --- Application State ---

struct ExampleApp {
    // Settings
    theme: iced::Theme,

    // Customizable chart Data
    series: Vec<Series>,
    labels: Vec<String>,

    // Chart State (Persistent)
    chart_state: State,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchTheme(iced::Theme),

    // Chart Actions
    SyncChart,
    AddLineSeries,
    AddBarSeries,
    AddRandomDataToSeries(usize),
    ClearData,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let app = Self {
            theme: iced::Theme::Dark,
            series: Vec::new(),
            labels: Vec::new(),
            chart_state: State::new(),
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTheme(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::SyncChart => {
                self.chart_state.sync(&self.series, &self.labels);
                Task::none()
            }
            Message::AddLineSeries => {
                let name = format!("Line {}", self.series.len() + 1);
                let color = generate_pastel_color();

                let series = Series::Line(LineSeries::new(name, Vec::new(), color).markers(true));
                self.series.push(series);
                Task::done(Message::SyncChart)
            }
            Message::AddBarSeries => {
                let name = format!("Bar {}", self.series.len() + 1);
                let color = generate_pastel_color();

                let series = Series::Bar(BarSeries::new(name, Vec::new(), color));
                self.series.push(series);
                Task::done(Message::SyncChart)
            }
            Message::AddRandomDataToSeries(index) => {
                // Generate random value between 0 and 100
                let val = generate_random_value();

                // Generate a label for this index (e.g. "1", "2", "3")
                // In a real app, this might be a timestamp
                let label = format!("{}", self.labels.len() + 1);

                if let Some(target_series) = self.series.get_mut(index) {
                    let current_len = match target_series {
                        Series::Line(s) => s.values.len(),
                        Series::Bar(s) => s.values.len(),
                    };

                    // If this series is growing beyond the current label set, add a new label
                    if current_len >= self.labels.len() {
                        self.labels.push(label);
                    }

                    match target_series {
                        Series::Line(s) => s.push(val),
                        Series::Bar(s) => s.push(val),
                    }
                }

                Task::done(Message::SyncChart)
            }
            Message::ClearData => {
                self.series.clear();
                self.labels.clear();
                Task::done(Message::SyncChart)
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- Top Bar: Theme ---
        let theme_toggle = pick_list(iced::Theme::ALL, Some(&self.theme), Message::SwitchTheme)
            .width(Length::Fill);

        // --- Middle: Chart ---
        // We give the chart the majority of the space
        let chart = CombinedChart::new(&self.chart_state, &self.series)
            .view()
            .width(Length::Fill)
            .height(Length::Fill);

        let chart_container = container(chart)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|t| {
                container::Style::default()
                    .background(t.palette().background)
                    .border(iced::border::color(t.palette().text).width(1.0))
            })
            .padding(20);

        // --- Bottom: Controls & Series List ---

        // 1. Global Controls
        let controls = row![
            button("Add Line Series")
                .on_press(Message::AddLineSeries)
                .width(Length::Fill),
            button("Add Bar Series")
                .on_press(Message::AddBarSeries)
                .width(Length::Fill),
            button("Clear All")
                .on_press(Message::ClearData)
                .style(button::danger)
                .width(Length::Fill),
        ]
        .spacing(10);

        // 2. Series List (Rows for each series)
        let series_rows = self.series.iter().enumerate().map(|(i, s)| {
            let (name, color, len) = match s {
                Series::Line(l) => (&l.name, l.color, l.values.len()),
                Series::Bar(b) => (&b.name, b.color, b.values.len()),
            };

            // Color indicator box
            let indicator = container(text(" "))
                .width(20)
                .height(20)
                .style(move |_| container::Style::default().background(color));

            container(
                row![
                    indicator,
                    text(name).width(Length::Fill),
                    text(format!("Points: {}", len)).size(12).width(100),
                    button("Add Random Data (0-100)")
                        .on_press(Message::AddRandomDataToSeries(i))
                        .style(button::secondary)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .padding(5)
            .style(|t| {
                container::Style::default()
                    .background(t.palette().background)
                    .border(iced::border::rounded(4))
            })
            .into()
        });

        let series_list = scrollable(column(series_rows).spacing(5)).height(Length::Fixed(200.0)); // Fixed height for the list area

        // Combine Layout
        column![
            theme_toggle,
            chart_container,
            column![controls, text("Active Series:").size(14), series_list]
                .spacing(10)
                .padding(10)
        ]
        .spacing(10)
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

fn generate_pastel_color() -> Color {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Color::from_rgb(
        rng.gen_range(0.4..0.8),
        rng.gen_range(0.4..0.8),
        rng.gen_range(0.4..0.9),
    )
}

fn generate_random_value() -> f64 {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(0.0..100.0)
}
