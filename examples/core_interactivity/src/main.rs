//! Minimal Interactive iced_aksel plot example using Cached.
use iced::{
    Element, Length, Theme,
    widget::{column, container, text},
};
use iced_aksel::{
    Axis, Cached, Chart, PlotPoint, State,
    axis::{self},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::Rectangle,
};

// -----------------------------------------------------------------------------
// 1. Application Entry
// -----------------------------------------------------------------------------
fn main() -> iced::Result {
    iced::application(TemplateApp::new, TemplateApp::update, TemplateApp::view)
        .theme(Theme::Light)
        .run()
}

// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------
struct TemplateApp {
    chart_state: State<&'static str, f64>,
    data: Cached<MyData>,
}

#[derive(Debug, Clone)]
enum Message {
    BarHovered(usize),
    BarClicked(usize),
    BackgroundHovered,
}

impl TemplateApp {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();
        state.set_axis(
            Self::X,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom),
        );
        state.set_axis(
            Self::Y,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Left),
        );

        let mock_data = MyData {
            bars: vec![
                Bar {
                    x: 10.0,
                    width: 10.0,
                    height: 40.0,
                    id: 101,
                },
                Bar {
                    x: 30.0,
                    width: 10.0,
                    height: 80.0,
                    id: 102,
                },
                Bar {
                    x: 50.0,
                    width: 10.0,
                    height: 60.0,
                    id: 103,
                },
                Bar {
                    x: 70.0,
                    width: 10.0,
                    height: 20.0,
                    id: 104,
                },
            ],
            hovered_bar_id: None,
            selected_bar_id: None,
        };

        (
            Self {
                chart_state: state,
                data: Cached::new(mock_data), // <--- Initialize the cache
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::BarHovered(id) => {
                // Calling .edit() gives mutable access and invalidates the cache for a redraw!
                self.data.edit().hovered_bar_id = Some(id);
            }
            Message::BarClicked(id) => self.data.edit().selected_bar_id = Some(id),
            Message::BackgroundHovered => self.data.edit().hovered_bar_id = None,
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, Self::X, Self::Y)
            .on_hover(|_| Message::BackgroundHovered);

        let header_text = match self.data.get().selected_bar_id {
            Some(id) => format!("You clicked Bar ID #{}!", id),
            None => "Click a bar!".to_string(),
        };

        column![
            text(header_text).size(24),
            container(chart).width(Length::Fill).height(Length::Fill)
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}

// -----------------------------------------------------------------------------
// 3. Data & Drawing Logic
// -----------------------------------------------------------------------------

struct Bar {
    x: f64,
    width: f64,
    height: f64,
    id: usize, // Semantic ID representing the data
}

struct MyData {
    bars: Vec<Bar>,
    hovered_bar_id: Option<usize>,
    selected_bar_id: Option<usize>,
}

impl PlotData<f64, Message> for MyData {
    fn draw(&self, plot: &mut Plot<f64, Message>, theme: &Theme) {
        let palette = theme.palette();

        for bar in &self.bars {
            let mut color = palette.primary;

            // Check state against the explicit ID, not the loop index!
            if Some(bar.id) == self.selected_bar_id {
                color = palette.danger;
            } else if Some(bar.id) == self.hovered_bar_id {
                color = palette.success;
            }

            plot.add_shape(
                Rectangle::corners(
                    PlotPoint::new(bar.x, 0.0),
                    PlotPoint::new(bar.x + bar.width, bar.height),
                )
                .id(bar.id) // <--- Tell the engine this shape's unique ID
                .fill(color)
                .on_hover(Message::BarHovered(bar.id))
                .on_click(Message::BarClicked(bar.id)),
            );
        }
    }
    // We completely deleted the manual `fn version()` here! Cached handles it automatically.
}
