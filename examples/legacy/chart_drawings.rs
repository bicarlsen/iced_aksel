//! Chart shape layer example

use iced::{
    Color, Element, Point, Task, Theme, event,
    widget::{Button, Slider, column, text},
};
use iced_extras::widget::chart::{
    self, Axis, Chart, Layer, PlotPoint, PlotRectangle, Position, State,
    drawing::{DrawingCache, Tool},
    scale::{Linear, Logarithmic},
    shape::{self, Length},
};

const LINEAR_AXIS_ID_X: &str = "linear_x";
const LINEAR_AXIS_ID_Y: &str = "linear_y";

type AxisId = &'static str;

#[derive(Debug, Clone)]
enum Message {
    UpdateChart,
    AddRandomDrawing,
    ChartHovered(Point),
    SubbedEvents(event::Event, event::Status),
}

struct ExampleApp {
    // Holds Scales
    state: State<AxisId>,

    // Holds Shapes that should be actively drawn
    plot_shapes: Layer<AxisId>,

    selected_tool: Tool,

    drawings: DrawingCache,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut state: State<AxisId> = State::new();

        let initial_bounds =
            PlotRectangle::from_points(PlotPoint::new(0., 0.), PlotPoint::new(100., 100.));

        // Use the constant for axis setup
        let axis_x = Axis::new(
            Linear::new(initial_bounds.min_x(), initial_bounds.max_x()),
            Position::Bottom,
        );
        let axis_y = Axis::new(
            Linear::new(initial_bounds.min_y(), initial_bounds.max_y()),
            Position::Left,
        );

        state.set_axis(LINEAR_AXIS_ID_X, axis_x);
        state.set_axis(LINEAR_AXIS_ID_Y, axis_y);

        let app = Self {
            state,
            selected_tool: Tool::Rectangle,
            plot_shapes: Layer::new(LINEAR_AXIS_ID_X, LINEAR_AXIS_ID_Y),
            drawings: DrawingCache::new(),
        };

        (app, Task::done(Message::UpdateChart))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UpdateChart => {
                // Take all drawings from handler and make shapes
                let plot_rect = &self
                    .state
                    .get_scales_plotrectangle(LINEAR_AXIS_ID_X, LINEAR_AXIS_ID_Y)
                    .unwrap();
                let drawings = self.drawings.get_visible_drawings(plot_rect);

                // Clear before adding, as we will redo all shapes
                self.plot_shapes.clear();

                for drawing in drawings {
                    self.plot_shapes.add_shape(*drawing);
                }

                Task::none()
            }

            Message::AddRandomDrawing => {
                let area = &self
                    .state
                    .get_scales_plotrectangle(LINEAR_AXIS_ID_X, LINEAR_AXIS_ID_Y)
                    .unwrap();
                let drawing = chart::drawing::Rectangle::random(area);
                self.drawings.add_drawing(drawing);

                Task::done(Message::UpdateChart)
            }

            Message::ChartHovered(norm_point) => {
                // TODO: Fix this offset, by making on_hover() function fit plot, not screen normalized values
                let y = 1.0 - norm_point.y;

                let x = self
                    .state
                    .get_axis(&LINEAR_AXIS_ID_X)
                    .unwrap()
                    .denormalize(norm_point.x);

                let y = self
                    .state
                    .get_axis(&LINEAR_AXIS_ID_Y)
                    .unwrap()
                    .denormalize(y);

                let hovered_drawings = self.drawings.hovered_drawings(&PlotPoint::new(x, y));

                for hovered in hovered_drawings {
                    println!("{:?}", hovered);
                }

                Task::none()
            }
            Message::SubbedEvents(event, status) => {
                println!("Event: {:?}, Status: {:?}", event, status);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .layer(&self.plot_shapes)
            .on_hover(Message::ChartHovered);

        let btn = Button::new("Add Random Drawing").on_press(Message::AddRandomDrawing);

        column![btn, chart].into()
    }

    const fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .antialiasing(true)
            .subscription(|app| {
                iced::event::listen_with(|e, s, id| Some(Message::SubbedEvents(e, s)))
            })
            .run()
    }
}

fn main() -> iced::Result {
    ExampleApp::run()
}

pub enum Tool {
    Cursor,
    Rectangle,
    Circle,
    HorizontalLine,
    VerticalLine,
    LineSegment,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Anchor {
    Horizontal(PlotPoint),
    Vertical(PlotPoint),
    Diagonal(PlotPoint),
    Free(PlotPoint),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RectangleStyle {
    pub(crate) outline: Option<Stroke>,
    pub(crate) fill: Color,
}

impl RectangleStyle {
    fn white() -> Self {
        Self {
            outline: None,
            fill: Color::WHITE,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    p1: PlotPoint,
    p2: PlotPoint,

    style: RectangleStyle,
}

impl Rectangle {
    // Create a random rectangle that is inside these outer bounds.
    pub fn random(bounds: &PlotRectangle) -> Self {
        let mut rng = rand::rng();
        let width = bounds.width() * 0.05;
        let height = bounds.height() * 0.05;

        // Randomize these
        // TODO: Fix this - Needs to use the bounds instead
        let x = rng.random_range(0.0..100.0);
        let y = rng.random_range(0.0..100.0);

        let p1 = PlotPoint::new(x - width, y - height);
        let p2 = PlotPoint::new(x + width, y + height);

        Self::from_points(p1, p2, RectangleStyle::white())
    }

    pub fn from_points(p1: PlotPoint, p2: PlotPoint, style: RectangleStyle) -> Self {
        Self { p1, p2, style }
    }

    fn aabb(self) -> AABB<PlotPoint> {
        AABB::from_corners(self.p1, self.p2)
    }

    fn as_shape(self) -> Shape {
        chart::shape::Shape::Rectangle(chart::shape::Rectangle::from_points(
            self.p1,
            self.p2,
            self.style.fill,
        ))
    }
}

#[derive(From, Clone, Copy, Debug)]
pub enum Drawing {
    Rectangle(Rectangle),
}

impl Drawing {
    pub fn aabb(&self) -> AABB<PlotPoint> {
        match self {
            Drawing::Rectangle(rect) => rect.aabb(),
        }
    }

    pub fn as_shape(&self) -> Shape {
        match self {
            Drawing::Rectangle(rect) => rect.as_shape(),
        }
    }
}

impl From<Drawing> for Shape {
    fn from(drawing: Drawing) -> Self {
        drawing.as_shape()
    }
}

new_key_type! {pub struct DrawingId;}

#[derive(Debug, Clone)]
struct SpatialIndex {
    id: DrawingId,
    aabb: AABB<PlotPoint>,
}

// Tell rstar how to read the AABB from our struct
impl RTreeObject for SpatialIndex {
    type Envelope = AABB<PlotPoint>;

    fn envelope(&self) -> Self::Envelope {
        self.aabb
    }
}

pub struct DrawingCache {
    drawings: SlotMap<DrawingId, Drawing>,
    spatial_tree: RTree<SpatialIndex>,
}

impl DrawingCache {
    pub fn new() -> Self {
        Self {
            drawings: SlotMap::with_key(),
            spatial_tree: RTree::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.drawings.len()
    }

    pub fn add_drawing<T: Into<Drawing>>(&mut self, drawing: T) {
        // 1. Create the drawing
        let id = self.drawings.insert(drawing.into());

        // 2. Calculate AABB and insert into R-Tree
        let drawing = &self.drawings[id];

        let index_entry = SpatialIndex {
            id,
            aabb: drawing.aabb(),
        };

        self.spatial_tree.insert(index_entry);
    }

    pub fn get_visible_drawings(&self, plot_rectangle: &PlotRectangle) -> Vec<&Drawing> {
        let viewport_aabb =
            AABB::from_corners(plot_rectangle.top_left(), plot_rectangle.bot_right());

        // This query is O(log N + k) - extremely fast
        self.spatial_tree
            .locate_in_envelope_intersecting(&viewport_aabb)
            .filter_map(|index_entry| {
                // Resolve ID to actual Drawing data
                self.drawings.get(index_entry.id)
            })
            .collect()
    }

    pub fn hovered_drawings(&self, mouse_position: &PlotPoint) -> Vec<&DrawingId> {
        // Create a small square around the cursor
        let mouse_square = AABB::from_point(*mouse_position);

        self.spatial_tree
            .locate_in_envelope_intersecting(&mouse_square)
            .map(|index_entry| &index_entry.id)
            .collect()
    }
}
