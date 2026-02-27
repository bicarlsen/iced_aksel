//! Minimal iced_aksel plot example.

use iced::widget::{button, row, text};
use iced::{
    Color, Element, Length, Point, Theme, Vector, mouse,
    widget::{column, container},
};
use iced_aksel::interaction::{Area, Id};
use iced_aksel::shape::Rectangle;
use iced_aksel::{
    Axis, Chart, Delta, DragEvent, Interaction, Measure, PlotPoint, ReleaseEvent, Shape, State,
    Stroke,
    axis::{self},
    plot::{Plot, PlotData},
    scale::Linear,
    shape,
};
use std::ops::Add;
// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum Tool {
    Select,
    Rectangle,
}

struct TemplateApp {
    tool: Tool,
    chart_state: State<&'static str, f64>,
    data: DrawingSystem,
}

#[derive(Debug, Clone)]
enum Message {
    ChartHovered,
    DrawingHovered(Id),
    LeftMouseRelease(ReleaseEvent<Point>),
    LeftMouseDragged(DragEvent<Delta>),
    MiddleMouseDragged(DragEvent<Delta>),
    ToolSelected(Tool),
}

impl TemplateApp {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        // -- Setup Axes --
        // Customizable: Change Linear to Logarithmic or adjust ranges here.
        let x_scale = Linear::new(0.0, 100.0);
        let y_scale = Linear::new(0.0, 100.0);

        let x_axis = Axis::new(x_scale, axis::Position::Bottom);
        let y_axis = Axis::new(y_scale, axis::Position::Left);

        state.set_axis(Self::X, x_axis);
        state.set_axis(Self::Y, y_axis);

        (
            Self {
                chart_state: state,
                data: DrawingSystem::new(),
                tool: Tool::Select,
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ChartHovered => {
                println!("ChartHovered!");
            }
            Message::DrawingHovered(id) => {
                println!("drawing hovered id: {:?}", id);
            }
            Message::ToolSelected(tool) => {
                self.tool = tool;
            }

            Message::LeftMouseDragged(drag_event) => {
                let delta = drag_event.delta;

                self.chart_state.axis_mut(&Self::X).pan(delta.x);
                self.chart_state.axis_mut(&Self::Y).pan(delta.y);
            }

            Message::MiddleMouseDragged(drag_event) => {
                let delta = drag_event.delta;

                self.chart_state.axis_mut(&Self::X).pan(delta.x);
                self.chart_state.axis_mut(&Self::Y).pan(delta.y);
            }

            Message::LeftMouseRelease(release_event) => {
                if release_event.was_dragging {
                    return;
                }

                if matches!(
                    release_event.click_kind,
                    Some(iced::advanced::mouse::click::Kind::Single)
                ) {
                    if self.tool == Tool::Rectangle {
                        let x_plot = self
                            .chart_state
                            .axis(&Self::X)
                            .denormalize(release_event.position.x);
                        let y_plot = self
                            .chart_state
                            .axis(&Self::Y)
                            .denormalize(release_event.position.y);

                        let p1 = PlotPoint::new(x_plot, y_plot);
                        let p2 = PlotPoint::new(x_plot + 15., y_plot + 15.);

                        // Create a rectangle
                        let rect = Drawing::Rectangle {
                            id: Id::unique(),
                            p1,
                            p2,
                            fill: Color::from_rgb(0., 0., 1.),
                            stroke: Stroke::new(Color::WHITE, Measure::Screen(1.)),
                        };

                        self.data.add_drawing(rect);
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let btn_row = row![
            button("Select").on_press(Message::ToolSelected(Tool::Select)),
            button("Rectangle").on_press(Message::ToolSelected(Tool::Rectangle)),
            text(format!["Tool Selected: {:?}", self.tool])
        ]
        .spacing(16.);

        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, Self::X, Self::Y)
            .on_drag(|v: DragEvent<Delta>| match v.button_held {
                mouse::Button::Left => Some(Message::LeftMouseDragged(v)),
                mouse::Button::Middle => Some(Message::MiddleMouseDragged(v)),
                _ => None,
            })
            .on_release(|v: ReleaseEvent<Point>| {
                if matches!(v.button, mouse::Button::Left) {
                    Some(Message::LeftMouseRelease(v))
                } else {
                    None
                }
            })
            .on_hover(|v| Some(Message::ChartHovered));

        column![btn_row, chart]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// -----------------------------------------------------------------------------
// 3. Data & Drawing Logic
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Drawing {
    Rectangle {
        id: Id,
        p1: PlotPoint,
        p2: PlotPoint,
        fill: Color,
        stroke: Stroke<f64>,
    },
}

impl Drawing {
    fn add_to_plot(self, plot: &mut Plot<f64, Message>) {
        match self {
            Drawing::Rectangle {
                id,
                p1,
                p2,
                fill,
                stroke,
            } => {
                let shape = shape::Rectangle::corners(p1, p2).fill(fill).stroke(stroke);
                // println!("{:?}", rectangle_plot(p1, p2));
                // println!("{:?} {:?}", p1, p2);

                // let interaction = Interaction::new(id, rectangle_plot(p1, p2))
                //     .on_hover(|id| Some(Message::DrawingHovered(id)))
                //     .on_drag(|_, _| None);

                // plot.add_interaction(interaction);
                plot.render(shape);
            }
        }
    }
}

struct DrawingSystem {
    drawings: Vec<Drawing>,
    ghost: Option<(Tool, Vec<Point>)>,
}

impl DrawingSystem {
    fn new() -> Self {
        Self {
            drawings: vec![],
            ghost: None,
        }
    }

    fn add_drawing(&mut self, drawing: Drawing) {
        self.drawings.push(drawing);
    }
}

impl PlotData<f64, Message> for DrawingSystem {
    fn draw(&self, plot: &mut Plot<f64, Message>, _theme: &Theme) {
        for drawing in self.drawings.clone().into_iter() {
            drawing.add_to_plot(plot);
        }
    }
}

// -----------------------------------------------------------------------------
// 1. Application Entry
// -----------------------------------------------------------------------------
fn main() -> iced::Result {
    iced::application(TemplateApp::new, TemplateApp::update, TemplateApp::view)
        .theme(Theme::Dark)
        .run()
}

/// Creates a rectangle area in pure plot data coordinates from two diagonal points.
pub fn rectangle_plot(p1: PlotPoint<f64>, p2: PlotPoint<f64>) -> Area<f64> {
    // 1. Find the anchor point (usually the minimum x and y)
    // We use a simple if/else because floating point numbers (f32/f64)
    // only implement PartialOrd, not Ord.
    let (min_x, max_x) = if p1.x < p2.x {
        (p1.x, p2.x)
    } else {
        (p2.x, p1.x)
    };
    let (min_y, max_y) = if p1.y < p2.y {
        (p1.y, p2.y)
    } else {
        (p2.y, p1.y)
    };

    // 2. Calculate the width and height in data space
    let w = max_x - min_x;
    let h = max_y - min_y;

    // 3. Construct the Rect using the Measure::Plot wrapper
    Area::Rect {
        x: min_x,
        y: min_y,
        width: Measure::Plot(w),
        height: Measure::Plot(h),
    }
}
