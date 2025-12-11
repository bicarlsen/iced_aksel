use aksel::{PlotPoint, scale::Linear};
use iced::{
    Color, Element, Length, Theme,
    widget::{checkbox, column, container, pick_list, row, slider, text},
};
use iced_aksel::{
    Axis, Chart, Measure, State, Stroke,
    axis::{self, GridLine, LabelDecision, TickLine},
    plot::{Plot, PlotData},
    shape::Polyline,
};

// -----------------------------------------------------------------------------
// Application Entry
// -----------------------------------------------------------------------------

pub fn main() -> iced::Result {
    iced::application(AxesExample::new, AxesExample::update, AxesExample::view)
        .title("Axes Configuration")
        .theme(Theme::Dark)
        .antialiasing(true)
        .run()
}

// -----------------------------------------------------------------------------
// Application State
// -----------------------------------------------------------------------------

pub struct AxesExample {
    chart_state: State<&'static str, f64>,
    data: DataLayer,
    settings: AxisSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickStyle {
    Simple,
    OnlyMajor,
}

impl std::fmt::Display for TickStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TickStyle::Simple => write!(f, "Simple"),
            TickStyle::OnlyMajor => write!(f, "Only Major"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AxisSettings {
    x_visible: bool,
    y_visible: bool,
    y_position_right: bool,
    show_grid: bool,
    show_cursor: bool,
    sparse_labels: bool,
    skip_overlap: bool,
    tick_style: TickStyle,
    thickness: f32,
    label_spacing: f32,
}

impl Default for AxisSettings {
    fn default() -> Self {
        Self {
            x_visible: true,
            y_visible: true,
            y_position_right: false,
            show_grid: true,
            show_cursor: true,
            sparse_labels: false,
            skip_overlap: true,
            tick_style: TickStyle::Simple,
            thickness: 30.0,
            label_spacing: 5.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleX(bool),
    ToggleY(bool),
    ToggleYSide(bool),
    ToggleGrid(bool),
    ToggleCursor(bool),
    ToggleSparse(bool),
    ToggleSkipOverlap(bool),
    TickStyleChanged(TickStyle),
    ThicknessChanged(f32),
    SpacingChanged(f32),
}

impl AxesExample {
    const X_ID: &'static str = "x";
    const Y_ID: &'static str = "y";

    fn new() -> (Self, iced::Task<Message>) {
        let mut app = Self {
            chart_state: State::new(),
            data: DataLayer::new(),
            settings: AxisSettings::default(),
        };
        app.rebuild_axes();
        (app, iced::Task::none())
    }

    fn rebuild_axes(&mut self) {
        // We abstract the creation logic into a helper function to keep this clean.
        // This makes it easy to see we are creating two axes with slightly different configs.

        let x_axis = self.create_axis(AxisKind::X);
        let y_axis = self.create_axis(AxisKind::Y);

        self.chart_state.set_axis(Self::X_ID, x_axis);
        self.chart_state.set_axis(Self::Y_ID, y_axis);
    }

    /// This function demonstrates the "Decision Flow" of configuring an Axis.
    fn create_axis(&self, kind: AxisKind) -> Axis<f64> {
        let s = self.settings;

        // 1. Define Base Properties (Scale & Position)
        let (scale, position) = match kind {
            AxisKind::X => (Linear::new(0.0, 100.0), axis::Position::Bottom),
            AxisKind::Y => (
                Linear::new(-50.0, 50.0),
                if s.y_position_right {
                    axis::Position::Right
                } else {
                    axis::Position::Left
                },
            ),
        };

        // 2. Start the Builder
        let mut axis = Axis::new(scale, position)
            .with_thickness(s.thickness)
            .with_label_spacing(s.label_spacing)
            .without_grid(); // Start clean for the toggle logic below

        // 3. Apply Visibility
        let is_visible = match kind {
            AxisKind::X => s.x_visible,
            AxisKind::Y => s.y_visible,
        };
        if !is_visible {
            axis = axis.invisible();
        }

        // 4. Configure Label Policy (Collision Avoidance)
        if s.sparse_labels {
            // Option A: Custom Logic (e.g., Even/Odd filtering)
            axis = axis.with_custom_label_policy(move |ctx| {
                let val = ctx.tick.value as i32;
                let render = match kind {
                    AxisKind::X => val % 2 == 0,  // X: Even numbers only
                    AxisKind::Y => val % 10 == 0, // Y: Multiples of 10 only
                };

                if render {
                    LabelDecision::Render
                } else {
                    LabelDecision::Skip
                }
            });
        } else if s.skip_overlap {
            // Option B: Automatic Collision Detection
            axis = axis.skip_overlapping_labels(10.0);
        }

        // 5. Configure Visuals (Ticks & Grid)
        axis = Self::apply_visuals(axis, s);

        // 6. Configure Interactive Elements (Cursor Labels)
        if s.show_cursor {
            axis = axis.with_cursor_formatter(|val| {
                Some(axis::Label {
                    content: format!("{:.1}", val),
                    size: 10.into(),
                })
            });
        }

        axis
    }

    /// Applies the TickRenderer and GridRenderer based on settings.
    fn apply_visuals(axis: Axis<f64>, s: AxisSettings) -> Axis<f64> {
        // A. Apply Tick Style
        let axis = match s.tick_style {
            TickStyle::Simple => axis
                .with_tick_renderer(|ctx| Some(TickLine::simple(format!("{:.0}", ctx.tick.value)))),
            TickStyle::OnlyMajor => axis.with_tick_renderer(|ctx| {
                // Logic: Only draw tick if level is 0 (Major)
                if ctx.tick.level == 0 {
                    Some(TickLine {
                        thickness: 1.5.into(),
                        length: 8.0.into(),
                        label: Some(axis::Label {
                            content: format!("{:.0}", ctx.tick.value),
                            size: 10.into(),
                        }),
                    })
                } else {
                    None // Skip minor ticks entirely
                }
            }),
        };

        // B. Apply Grid
        if s.show_grid {
            axis.with_grid_renderer(|tick| {
                if tick.level == 0 {
                    Some(GridLine {
                        thickness: if tick.level == 0 {
                            1.0.into()
                        } else {
                            0.5.into()
                        },
                    })
                } else {
                    None
                }
            })
        } else {
            axis
        }
    }

    fn update(&mut self, message: Message) -> iced::Task<Message> {
        match message {
            Message::ToggleX(v) => self.settings.x_visible = v,
            Message::ToggleY(v) => self.settings.y_visible = v,
            Message::ToggleYSide(v) => self.settings.y_position_right = v,
            Message::ToggleGrid(v) => self.settings.show_grid = v,
            Message::ToggleCursor(v) => self.settings.show_cursor = v,
            Message::ToggleSparse(v) => self.settings.sparse_labels = v,
            Message::ToggleSkipOverlap(v) => self.settings.skip_overlap = v,
            Message::TickStyleChanged(v) => self.settings.tick_style = v,
            Message::ThicknessChanged(v) => self.settings.thickness = v,
            Message::SpacingChanged(v) => self.settings.label_spacing = v,
        }
        self.rebuild_axes();
        iced::Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.chart_state).plot_data(&self.data, Self::X_ID, Self::Y_ID);

        let controls = column![
            text("Layout & Visibility")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            row![
                checkbox(self.settings.x_visible).on_toggle(Message::ToggleX),
                text("X Axis")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.y_visible).on_toggle(Message::ToggleY),
                text("Y Axis")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.y_position_right).on_toggle(Message::ToggleYSide),
                text("Y Axis on Right")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.show_grid).on_toggle(Message::ToggleGrid),
                text("Grid Lines")
            ]
            .spacing(10),
            text("Labels & Ticks")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            row![
                checkbox(self.settings.show_cursor).on_toggle(Message::ToggleCursor),
                text("Cursor Labels")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.skip_overlap).on_toggle(Message::ToggleSkipOverlap),
                text("Skip Overlapping")
            ]
            .spacing(10),
            row![
                checkbox(self.settings.sparse_labels).on_toggle(Message::ToggleSparse),
                text("Sparse Policy (Custom)")
            ]
            .spacing(10),
            text("Tick Style")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            pick_list(
                &[TickStyle::Simple, TickStyle::OnlyMajor][..],
                Some(self.settings.tick_style),
                Message::TickStyleChanged
            ),
            text(format!("Thickness: {:.0}px", self.settings.thickness))
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            slider(
                10.0..=100.0,
                self.settings.thickness,
                Message::ThicknessChanged
            ),
            text(format!("Spacing: {:.0}px", self.settings.label_spacing))
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7)),
            slider(
                0.0..=20.0,
                self.settings.label_spacing,
                Message::SpacingChanged
            ),
        ]
        .spacing(12)
        .padding(20)
        .width(250);

        row![
            container(chart)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(20),
            container(controls).height(Length::Fill).style(|t: &Theme| {
                container::Style::default().background(t.extended_palette().background.weak.color)
            })
        ]
        .into()
    }
}

// Helper enum to distinguish X/Y during creation
enum AxisKind {
    X,
    Y,
}

struct DataLayer {
    points: Vec<PlotPoint<f64>>,
}

impl DataLayer {
    fn new() -> Self {
        let points = (0..=100)
            .map(|i| {
                let x = i as f64;
                let y = (x * 0.1).sin() * 40.0;
                PlotPoint::new(x, y)
            })
            .collect();
        Self { points }
    }
}

impl PlotData<f64, iced::Renderer, Theme> for DataLayer {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        plot.add_shape(Polyline::new(
            self.points.clone(),
            Stroke::new(theme.palette().primary, Measure::Screen(2.0)),
        ));
    }
}
