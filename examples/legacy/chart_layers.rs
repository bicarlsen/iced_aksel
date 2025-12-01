//! Chart shape layer example

use aksel::{PlotPoint, PlotRect, scale::Linear};
use iced::{
    Color, Element, Task, Theme,
    widget::{Button, Slider, TextInput, column, row},
};
use iced_extras::widget::chart::{
    Axis, Chart, DragDelta, Layer, Position, State,
    axis::{self, TickLine},
    shape::{self, Label, Length},
};
use rand::Rng;

const LINEAR_AXIS_ID_X: &str = "linear_x";
const LINEAR_AXIS_ID_Y: &str = "linear_y";

/// Define the plot's boundaries once.
const PLOT_BOUNDS: PlotRect = PlotRect {
    x: 0.,
    y: 0.,
    width: 100.,
    height: 100.,
};

type AxisId = &'static str;
type Float = f64;

#[derive(Debug, Clone)]
enum Message {
    LabelInputConfirmed,
    LabelInputChanged(String),
    ShapeLayerConfirmed(u32),
    ShapeLayerSliderChanged(u32),
    PanChart(DragDelta),
    ChartClicked(iced::Point),
}

struct ExampleApp {
    state: State<AxisId, Float>,

    // Layer buffer. This is all `Shape` and the axis it belongs to
    layers: Vec<Layer<AxisId, Float>>,

    // Widget values
    label_input_text: String,
    shape_count: u32,
}

impl ExampleApp {
    fn init() -> (Self, Task<Message>) {
        let mut state = State::new();

        let axis_x = Axis::new(
            Linear::new(PLOT_BOUNDS.x, PLOT_BOUNDS.x + PLOT_BOUNDS.width),
            Position::Bottom,
        )
        .with_tick_renderer(|tick| {
            let length = match tick.level {
                0 => 10.0,
                _ => 5.0,
            }
            .into();
            Some(TickLine {
                thickness: iced::Pixels(2.0),
                color: Color::WHITE,
                label: Some(axis::Label {
                    color: Color::WHITE,
                    size: iced::Pixels(8.0),
                    content: tick.value.to_string(),
                }),
                length,
            })
        });
        let axis_y = Axis::new(
            Linear::new(PLOT_BOUNDS.y, PLOT_BOUNDS.y + PLOT_BOUNDS.height),
            Position::Left,
        );

        state.set_axis(LINEAR_AXIS_ID_X, axis_x);
        state.set_axis(LINEAR_AXIS_ID_Y, axis_y);

        let app = Self {
            state,
            layers: vec![Layer::new(LINEAR_AXIS_ID_X, LINEAR_AXIS_ID_Y)],
            label_input_text: String::new(),
            shape_count: 0,
        };

        // Start with 500 shapes so the app isn't empty
        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::PanChart(delta) => {
                self.state
                    .get_axis_mut(&LINEAR_AXIS_ID_X)
                    .unwrap()
                    .scale_mut()
                    .pan(delta.x as f64);
                self.state
                    .get_axis_mut(&LINEAR_AXIS_ID_Y)
                    .unwrap()
                    .scale_mut()
                    .pan(delta.y as f64);
            }
            Message::ChartClicked(point) => {
                println!("Clicked: {point:?}");
            }
            // Labels
            Message::LabelInputChanged(s) => {
                self.label_input_text = s;
            }
            // Creates a new Label layer at each confirmation
            Message::LabelInputConfirmed => {
                let plotpoint = generate_random_plotpoint(dbg!(
                    self.state
                        .get_scales_plotrectangle(LINEAR_AXIS_ID_X, LINEAR_AXIS_ID_Y)
                        .unwrap()
                ));
                let label = Label::simple(self.label_input_text.clone(), plotpoint, Color::WHITE);
                let mut layer = Layer::new(LINEAR_AXIS_ID_X, LINEAR_AXIS_ID_Y);
                layer.add_shape(label);
                self.layers.push(layer);
            }

            // Shapes
            Message::ShapeLayerConfirmed(count) => {
                // Pass the constant to the generator
                let shapes = generate_random_pattern(count, PLOT_BOUNDS);
                let mut layer = Layer::new(LINEAR_AXIS_ID_X, LINEAR_AXIS_ID_Y);
                layer.add_shapes(shapes);
                self.layers.push(layer);
            }
            Message::ShapeLayerSliderChanged(count) => {
                self.shape_count = count;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .layers(&self.layers)
            .on_drag(Message::PanChart)
            .on_click(Message::ChartClicked);

        let shape_amount_slider = Slider::new(0..=100_000, self.shape_count, |value| {
            Message::ShapeLayerSliderChanged(value)
        });
        let shape_layer_confirm_btn = Button::new("Add New Shape Layer")
            .on_press(Message::ShapeLayerConfirmed(self.shape_count));

        let input = TextInput::new("", &self.label_input_text).on_input(Message::LabelInputChanged);
        let input_confirm_btn =
            Button::new("Add Label Randomly").on_press(Message::LabelInputConfirmed);

        let shape_row = row![shape_amount_slider, shape_layer_confirm_btn];
        let input_row = row![input, input_confirm_btn];

        column![chart, input_row, shape_row].into()
    }

    const fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .antialiasing(true)
            .run()
    }
}

fn main() -> iced::Result {
    ExampleApp::run()
}

fn generate_random_plotpoint(within: PlotRect) -> PlotPoint {
    let mut rng = rand::rng();
    let x = rng.random_range(within.x..within.x + within.width);
    let y = rng.random_range(within.y..within.y + within.height);
    PlotPoint { x, y }
}

/// Generates a `Vec<Shape>` containing a randomized pattern of shapes
/// with colors themed by their horizontal position.
fn generate_random_pattern(count: u32, bounds: PlotRect) -> Vec<shape::Shape<Float>> {
    // --- Define the simulation boundaries ---
    let min_x = bounds.x;
    let max_x = bounds.x + bounds.width;
    let min_y = bounds.y;
    let max_y = bounds.y + bounds.height;

    // --- Get a thread-local random number generator ---
    let mut rng = rand::rng();

    // --- Generate `count` random shapes ---
    (0..count)
        .filter_map(|_| {
            // 1. Generate a random position
            let x = rng.random_range(min_x..max_x);
            let y = rng.random_range(min_y..max_y);
            let pos = PlotPoint::new(x, y);

            // --- MODIFIED SECTION: Themed Color ---

            // 2. Generate a "themed" color based on X-position
            // We'll map the x-coordinate to a specific slice of the color wheel.
            // Example: A "cool" theme from Blue (220) to Purple (280).
            const HUE_START: f64 = 220.0;
            const HUE_END: f64 = 280.0;
            let hue_range = HUE_END - HUE_START;

            // Calculate how far across the screen 'x' is (a value from 0.0 to 1.0)
            let x_percent = (x - min_x) / bounds.width;

            // Determine the hue based on that percentage
            let hue = x_percent.mul_add(hue_range, HUE_START);

            // We still randomize saturation and lightness to keep it interesting!
            // Saturation: 50% to 100% (always colorful)
            let saturation = rng.random_range(0.5..1.0);
            // Lightness: 40% to 70% (avoids pure black/white)
            let lightness = rng.random_range(0.4..0.7);

            // --- END MODIFIED SECTION ---

            // Manually convert HSL to RGB
            let (r, g, b) = hsl_to_rgb(hue as f32, saturation, lightness);
            let color = Color::from_rgba(r, g, b, 1.0);

            // 3. Generate a random size
            let size_val = rng.random_range(2.0..8.0);
            let shape_size = Length::Screen(size_val);

            if pos.x < 0.0 || pos.y < 0.0 {
                println!("{pos:?}");
            };

            // Note: Your doc comment mentions circles, but this only creates rectangles (squares).
            // This is perfectly fine, just pointing it out!
            shape::Rectangle::from_center(pos, shape_size, shape_size, color).map(Into::into)
            // shape::Ellipse::new(pos, shape_size, shape_size, color).into()
        })
        .collect()
}

/// Converts HSL (Hue, Saturation, Lightness) to RGB.
/// H is in [0, 360], S and L are in [0, 1].
/// Returns (r, g, b) as floats in [0, 1].
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - 2.0f32.mul_add(l, -1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&h) {
        (c, x, 0.0)
    } else if (60.0..120.0).contains(&h) {
        (x, c, 0.0)
    } else if (120.0..180.0).contains(&h) {
        (0.0, c, x)
    } else if (180.0..240.0).contains(&h) {
        (0.0, x, c)
    } else if (240.0..300.0).contains(&h) {
        (x, 0.0, c)
    } else {
        // 300.0..360.0
        (c, 0.0, x)
    };

    (r_prime + m, g_prime + m, b_prime + m)
}
