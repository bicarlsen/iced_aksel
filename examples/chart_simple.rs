use aksel::scale::Linear;
use iced::{Element, Task, Theme};
use iced_aksel::{Axis, Chart, State, axis};

const X_ID: &str = "linear_x";
const Y_ID: &str = "linear_y";

type AxisId = &'static str;

fn main() -> iced::Result {
    ExampleApp::run()
}

// --- Application State ---

struct ExampleApp {
    state: State<AxisId, f64>,
}

#[derive(Debug, Clone)]
enum Message {}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut chart_state = State::new();

        // Initialize axes 0-100 on both axis
        chart_state.set_axis(
            X_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom),
        );
        chart_state.set_axis(
            Y_ID,
            Axis::new(Linear::new(0.0, 100.0), axis::Position::Right),
        );
        (Self { state: chart_state }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state);
        chart.into()
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Theme::Dark)
            .run()
    }
}
