//! iced_aksel Drawing Library Example (With Zoom & Z-Index Testing)
use iced::{
    Element, Length, Point, Theme,
    mouse::ScrollDelta,
    widget::{button, column, container, row, text},
};
use iced_aksel::{
    Axis, Cached, Chart, PlotPoint, State,
    axis::{self},
    plot::{DragDelta, Plot, PlotData},
    scale::Linear,
    shape::Rectangle,
};

fn main() -> iced::Result {
    iced::application(DrawingApp::new, DrawingApp::update, DrawingApp::view)
        .theme(Theme::CatppuccinMocha)
        .run()
}

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    Interact, // Pan and move shapes
    AddShape, // Click background to add shapes
}

// -----------------------------------------------------------------------------
// App State
// -----------------------------------------------------------------------------
struct DrawingApp {
    chart_state: State<&'static str, f64>,
    data: Cached<DrawingData>,

    x_range: (f64, f64),
    y_range: (f64, f64),

    mode: AppMode,
    next_id: usize, // Keeps IDs unique as we spawn new shapes!
}

#[derive(Debug, Clone)]
enum Message {
    // UI Controls
    SetMode(AppMode),

    // Shape Interactions
    ShapeHovered(usize),
    ShapePressed(usize),
    ShapeClicked(usize),

    // Global Interactions
    BackgroundHovered,
    BackgroundPressed,
    BackgroundClicked(Point),
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),
    DragEnded,
}

impl DrawingApp {
    const X: &'static str = "x";
    const Y: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();
        let initial_x = (0.0, 100.0);
        let initial_y = (0.0, 100.0);

        state.set_axis(
            Self::X,
            Axis::new(
                Linear::new(initial_x.0, initial_x.1),
                axis::Position::Bottom,
            ),
        );
        state.set_axis(
            Self::Y,
            Axis::new(Linear::new(initial_y.0, initial_y.1), axis::Position::Left),
        );

        let mock_data = DrawingData {
            rects: vec![
                RectShape {
                    x: 20.0,
                    y: 20.0,
                    w: 30.0,
                    h: 30.0,
                    id: 1,
                },
                RectShape {
                    x: 40.0,
                    y: 40.0,
                    w: 30.0,
                    h: 30.0,
                    id: 2,
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
                x_range: initial_x,
                y_range: initial_y,
                mode: AppMode::Interact,
                next_id: 3,
            },
            iced::Task::none(),
        )
    }

    fn update_axes(&mut self) {
        self.chart_state.set_axis(
            Self::X,
            Axis::new(
                Linear::new(self.x_range.0, self.x_range.1),
                axis::Position::Bottom,
            ),
        );
        self.chart_state.set_axis(
            Self::Y,
            Axis::new(
                Linear::new(self.y_range.0, self.y_range.1),
                axis::Position::Left,
            ),
        );
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SetMode(mode) => self.mode = mode,

            // --- Shape Interactions ---
            Message::ShapeHovered(id) => self.data.edit().hovered_id = Some(id),
            Message::ShapeClicked(id) => self.data.edit().selected_id = Some(id),
            Message::ShapePressed(id) => {
                if self.mode == AppMode::Interact {
                    let data = self.data.edit();
                    data.selected_id = Some(id);
                    data.dragging_id = Some(id);
                }
            }

            // --- Background Interactions ---
            Message::BackgroundHovered => self.data.edit().hovered_id = None,
            Message::BackgroundPressed => {
                let data = self.data.edit();
                data.selected_id = None;
                data.dragging_id = None;
            }

            Message::BackgroundClicked(pt) => {
                if self.mode == AppMode::AddShape {
                    // Reverse-project the normalized Point (0.0-1.0) into Data Space!
                    let x_span = self.x_range.1 - self.x_range.0;
                    let y_span = self.y_range.1 - self.y_range.0;

                    let data_x = self.x_range.0 + (pt.x as f64 * x_span);
                    let data_y = self.y_range.0 + (pt.y as f64 * y_span);

                    // Drop a new 15x15 box centered on the mouse
                    self.data.edit().rects.push(RectShape {
                        x: data_x - 7.5,
                        y: data_y - 7.5,
                        w: 15.0,
                        h: 15.0,
                        id: self.next_id,
                    });
                    self.next_id += 1;
                }
            }

            // --- Drag & Zoom Routing ---
            Message::ChartDragged(delta) => {
                let data = self.data.edit();
                data.hovered_id = None;

                let x_width = self.x_range.1 - self.x_range.0;
                let y_height = self.y_range.1 - self.y_range.0;
                let data_dx = (delta.x as f64) * x_width;
                let data_dy = (delta.y as f64) * y_height;

                if let Some(drag_id) = data.dragging_id {
                    // WE ARE DRAGGING A SHAPE
                    if let Some(rect) = data.rects.iter_mut().find(|r| r.id == drag_id) {
                        rect.x -= data_dx;
                        rect.y -= data_dy;
                    }
                } else if self.mode == AppMode::Interact {
                    // WE ARE PANNING THE CHART
                    self.x_range.0 += data_dx;
                    self.x_range.1 += data_dx;
                    self.y_range.0 += data_dy;
                    self.y_range.1 += data_dy;
                    self.update_axes();
                }
            }
            Message::DragEnded => {
                self.data.edit().dragging_id = None;
            }
            Message::ChartScrolled(pt, delta) => {
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

                let x_span = self.x_range.1 - self.x_range.0;
                let y_span = self.y_range.1 - self.y_range.0;

                // Find the exact data coordinate the mouse is hovering over
                let cursor_data_x = self.x_range.0 + (pt.x as f64 * x_span);
                let cursor_data_y = self.y_range.0 + (pt.y as f64 * y_span);

                let new_x_span = x_span * zoom_factor;
                let new_y_span = y_span * zoom_factor;

                // Contract/Expand the bounds around the cursor's location!
                self.x_range.0 = cursor_data_x - (pt.x as f64 * new_x_span);
                self.x_range.1 = cursor_data_x + ((1.0 - pt.x as f64) * new_x_span);

                self.y_range.0 = cursor_data_y - (pt.y as f64 * new_y_span);
                self.y_range.1 = cursor_data_y + ((1.0 - pt.y as f64) * new_y_span);

                self.data.edit().hovered_id = None; // Kill hovers during zoom
                self.update_axes();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.data, Self::X, Self::Y)
            .on_hover(|_| Message::BackgroundHovered)
            .on_press(|_| Message::BackgroundPressed)
            .on_click(Message::BackgroundClicked) // Detects full click to add shape
            .on_drag(Message::ChartDragged)
            .on_drag_end(|| Message::DragEnded)
            .on_scroll(Message::ChartScrolled); // <--- BIND SCROLL

        // UI Header with Mode Buttons
        let mode_text = match self.mode {
            AppMode::Interact => "Mode: Interact & Pan",
            AppMode::AddShape => "Mode: Click to Add Shape",
        };

        let controls = row![
            button("Interact Mode").on_press(Message::SetMode(AppMode::Interact)),
            button("Add Shape Mode").on_press(Message::SetMode(AppMode::AddShape)),
            text(mode_text).size(20)
        ]
        .spacing(20);

        column![
            controls,
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
    id: usize,
}
struct DrawingData {
    rects: Vec<RectShape>,
    hovered_id: Option<usize>,
    selected_id: Option<usize>,
    dragging_id: Option<usize>,
}

impl PlotData<f64, Message> for DrawingData {
    fn draw(&self, plot: &mut Plot<f64, Message>, theme: &Theme) {
        let palette = theme.palette();

        // Shapes draw in order of the array.
        // Newer shapes are pushed to the end, drawing ON TOP of older shapes.
        for rect in &self.rects {
            let mut color = palette.primary;
            if Some(rect.id) == self.selected_id {
                color = palette.danger;
            } else if Some(rect.id) == self.hovered_id {
                color = palette.success;
            }

            plot.add_shape(
                Rectangle::corners(
                    PlotPoint::new(rect.x, rect.y),
                    PlotPoint::new(rect.x + rect.w, rect.y + rect.h),
                )
                .id(rect.id)
                .fill(color)
                .on_hover(Message::ShapeHovered(rect.id))
                .on_press(Message::ShapePressed(rect.id))
                .on_click(Message::ShapeClicked(rect.id)),
            );
        }
    }
}
