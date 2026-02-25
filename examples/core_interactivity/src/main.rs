//! iced_aksel Drawing Library Example (With Zoom & Z-Index Testing)
use iced::{
    Element, Length, Point, Theme, keyboard,
    mouse::{self, ScrollDelta},
    widget::{column, container, text},
};
use iced_aksel::{
    Axis, Cached, Chart, Interaction, PlotPoint, State,
    axis::{self},
    interaction,
    plot::{DragDelta, Plot, PlotData},
    scale::Linear,
    shape::Rectangle,
};

fn main() -> iced::Result {
    iced::application(DrawingApp::new, DrawingApp::update, DrawingApp::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

// -----------------------------------------------------------------------------
// App State
// -----------------------------------------------------------------------------
struct DrawingApp {
    chart_state: State<&'static str, f64>,
    data: Cached<DrawingData>,
}

#[derive(Debug, Clone)]
enum Message {
    // Shape Interactions
    ShapeHovered(interaction::Id),
    ShapeDragged(interaction::Id, DragDelta),
    ShapeSelected(interaction::Id),

    // Global Interactions
    AddShape(Point),
    DeleteShape(interaction::Id),
    BackgroundHovered,
    BackgroundPressed,
    BackgroundClicked(Point),
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),
}

impl DrawingApp {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        state.set_axis(
            Self::X,
            Axis::new(Linear::new(0., 100.), axis::Position::Bottom),
        );
        state.set_axis(
            Self::Y,
            Axis::new(Linear::new(0., 100.), axis::Position::Left),
        );

        let mock_data = DrawingData {
            rects: vec![
                RectShape {
                    x: 20.0,
                    y: 20.0,
                    w: 30.0,
                    h: 30.0,
                    id: interaction::Id::unique(),
                },
                RectShape {
                    x: 40.0,
                    y: 40.0,
                    w: 30.0,
                    h: 30.0,
                    id: interaction::Id::unique(),
                },
            ],
            hovered_id: None,
            selected_id: None,
            dragging_id: None,
        };

        (
            Self {
                chart_state: state,
                data: Cached::new(mock_data),
            },
            iced::Task::none(),
        )
    }

    fn update(&mut self, message: Message) {
        match message {
            // --- Shape Interactions ---
            Message::ShapeHovered(id) => self.data.edit().hovered_id = Some(id),
            Message::ShapeDragged(id, delta) => {
                let data = self.data.edit();

                let (x_min, x_max) = self.chart_state.axis(&Self::X).domain();
                let (y_min, y_max) = self.chart_state.axis(&Self::Y).domain();

                let x_width = x_max - x_min;
                let y_height = y_max - y_min;

                // Convert normalized delta to data-space coordinates
                let dx = (delta.x as f64) * x_width;
                let dy = (delta.y as f64) * y_height;

                if let Some(rect) = data.rects.iter_mut().find(|r| r.id == id) {
                    rect.x += dx;
                    rect.y += dy;
                };
            }
            Message::AddShape(point) => {
                // Reverse-project the normalized Point (0.0-1.0) into data-space
                let data_x = self.chart_state.axis(&Self::X).denormalize(point.x);
                let data_y = self.chart_state.axis(&Self::Y).denormalize(point.y);

                self.data.edit().rects.push(RectShape {
                    x: data_x - 7.5,
                    y: data_y - 7.5,
                    w: 15.0,
                    h: 15.0,
                    id: interaction::Id::unique(),
                })
            }
            Message::DeleteShape(id) => {
                self.data.edit().rects.pop_if(|rect| rect.id == id);
            }
            Message::ShapeSelected(id) => {
                self.data.edit().selected_id = Some(id);
            }
            // Message::ShapeClicked(id) => self.data.edit().selected_id = Some(id),
            // Message::ShapePressed(id) => {
            //     if self.mode == AppMode::Interact {
            //         let data = self.data.edit();
            //         data.selected_id = Some(id);
            //         data.dragging_id = Some(id);
            //     }
            // }

            // --- Background Interactions ---
            Message::BackgroundHovered => self.data.edit().hovered_id = None,
            Message::BackgroundPressed => {
                let data = self.data.edit();
                data.selected_id = None;
                data.dragging_id = None;
            }

            Message::BackgroundClicked(pt) => {
                // Reverse-project the normalized Point (0.0-1.0) into data-space
                let data_x = self.chart_state.axis(&Self::X).denormalize(pt.x);
                let data_y = self.chart_state.axis(&Self::Y).denormalize(pt.y);

                // Drop a new 15x15 box centered on the mouse
                self.data.edit().rects.push(RectShape {
                    x: data_x - 7.5,
                    y: data_y - 7.5,
                    w: 15.0,
                    h: 15.0,
                    id: interaction::Id::unique(),
                });
            }

            // --- Drag & Zoom Routing ---
            Message::ChartDragged(delta) => {
                self.chart_state
                    .pan_axes(Self::X, Self::Y, delta.x, delta.y);
            }
            Message::ChartScrolled(point, delta) => {
                // Determine zoom factor (0.9 to zoom in, 1.1 to zoom out)
                let zoom_factor = match delta {
                    ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. } => {
                        if y > 0.0 {
                            0.9
                        } else {
                            1.1
                        }
                    }
                };

                self.chart_state
                    .axis_mut(&Self::X)
                    .zoom(zoom_factor, Some(point.x));
                self.chart_state
                    .axis_mut(&Self::Y)
                    .zoom(zoom_factor, Some(point.y));

                self.data.edit().hovered_id = None; // Kill hovers during zoom
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, Self::X, Self::Y)
            .on_hover(|_| Message::BackgroundHovered)
            .on_press(|_| Some(Message::BackgroundPressed))
            .on_release(|event| {
                (event.button == mouse::Button::Left)
                    .then_some(Message::BackgroundClicked(event.position))
            })
            .on_drag(Message::ChartDragged)
            .on_scroll(Message::ChartScrolled);

        column![
            text("Interactions Demo").size(30),
            text("Right click to new shapes").size(16),
            container(chart).width(Length::Fill).height(Length::Fill)
        ]
        .spacing(20)
        .padding(20)
        .into()
    }
}

// -----------------------------------------------------------------------------
// Data Structs
// -----------------------------------------------------------------------------
struct RectShape {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    id: interaction::Id,
}
struct DrawingData {
    rects: Vec<RectShape>,
    hovered_id: Option<interaction::Id>,
    selected_id: Option<interaction::Id>,
    dragging_id: Option<interaction::Id>,
}

impl PlotData<f64, Message> for DrawingData {
    fn draw(&self, plot: &mut Plot<f64, Message>, theme: &Theme) {
        let palette = theme.palette();

        // Shapes draw in order of the array.
        // Newer shapes are pushed to the end, drawing ON TOP of older shapes.
        for rect in &self.rects {
            let mut color = palette.primary;
            if Some(&rect.id) == self.selected_id.as_ref() {
                color = palette.danger;
            } else if Some(&rect.id) == self.hovered_id.as_ref() {
                color = palette.success;
            }

            let shape = Rectangle::corners(
                PlotPoint::new(rect.x, rect.y),
                PlotPoint::new(rect.x + rect.w, rect.y + rect.h),
            )
            .fill(color);

            plot.add_interaction(
                Interaction::new(rect.id.clone(), &shape)
                    .on_hover_with(Message::ShapeHovered)
                    .on_press_with(|id, event| match event.button {
                        mouse::Button::Left => Message::ShapeSelected(id),
                        mouse::Button::Right
                            if event.modifiers.contains(keyboard::Modifiers::SHIFT) =>
                        {
                            Message::DeleteShape(id)
                        }
                        mouse::Button::Right => Message::AddShape(event.position),
                        _ => unimplemented!(),
                    })
                    .on_drag_with(Message::ShapeDragged),
            );
            plot.render(shape);
        }
    }
}
