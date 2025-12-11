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
enum TickStyle {
    Simple,
    OnlyMajor, // Renamed from Dense
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
    sparse_labels: bool, // Custom Label Policy
    skip_overlap: bool,  // Standard Skip Policy
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
        let s = self.settings;

        // --- 1. X-AXIS ---
        let mut x_axis = Axis::new(Linear::new(0.0, 100.0), axis::Position::Bottom)
            .with_thickness(s.thickness)
            .with_label_spacing(s.label_spacing)
            .without_grid(); // Important: clear grid by default

        if !s.x_visible {
            x_axis = x_axis.invisible();
        }

        // Label Policy Logic
        if s.sparse_labels {
            // [Custom Policy]: Manually skip ticks (e.g., odd numbers)
            x_axis = x_axis.with_custom_label_policy(|ctx| {
                if ctx.tick.value as i32 % 2 == 0 {
                    LabelDecision::Render
                } else {
                    LabelDecision::Skip
                }
            });
        } else if s.skip_overlap {
            // [Standard Feature]: Skip labels that overlap
            x_axis = x_axis.skip_overlapping_labels(10.0);
        }

        // Cursor Formatter
        if s.show_cursor {
            x_axis = x_axis.with_cursor_formatter(|val| {
                Some(axis::Label {
                    content: format!("{:.1}", val),
                    size: 10.into(),
                })
            });
        }

        x_axis = Self::apply_tick_style(x_axis, s.tick_style);

        if s.show_grid {
            x_axis = x_axis.with_grid_renderer(|tick| {
                if tick.level == 0 {
                    Some(GridLine {
                        thickness: 1.0.into(),
                    })
                } else {
                    None
                }
            });
        }

        // --- 2. Y-AXIS ---
        let y_pos = if s.y_position_right {
            axis::Position::Right
        } else {
            axis::Position::Left
        };

        let mut y_axis = Axis::new(Linear::new(-50.0, 50.0), y_pos)
            .with_thickness(s.thickness)
            .with_label_spacing(s.label_spacing)
            .without_grid();

        if !s.y_visible {
            y_axis = y_axis.invisible();
        }

        // Label Policy Logic (Y Axis)
        if s.sparse_labels {
            y_axis = y_axis.with_custom_label_policy(|ctx| {
                // Skip ticks that aren't multiples of 10
                if ctx.tick.value as i32 % 10 == 0 {
                    LabelDecision::Render
                } else {
                    LabelDecision::Skip
                }
            });
        } else if s.skip_overlap {
            y_axis = y_axis.skip_overlapping_labels(10.0);
        }

        if s.show_cursor {
            y_axis = y_axis.with_cursor_formatter(|val| {
                Some(axis::Label {
                    content: format!("{:.1}", val),
                    size: 10.into(),
                })
            });
        }

        y_axis = Self::apply_tick_style(y_axis, s.tick_style);

        if s.show_grid {
            y_axis = y_axis.with_grid_renderer(|tick| {
                if tick.level == 0 {
                    Some(GridLine {
                        thickness: 0.5.into(),
                    })
                } else {
                    None
                }
            });
        }

        self.chart_state.set_axis(Self::X_ID, x_axis);
        self.chart_state.set_axis(Self::Y_ID, y_axis);
    }

    fn apply_tick_style(axis: Axis<f64>, style: TickStyle) -> Axis<f64> {
        match style {
            TickStyle::Simple => axis
                .with_tick_renderer(|ctx| Some(TickLine::simple(format!("{:.0}", ctx.tick.value)))),
            TickStyle::OnlyMajor => axis.with_tick_renderer(|ctx| {
                // Only render Major ticks (Level 0)
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
                    None
                }
            }),
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
