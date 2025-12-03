//! Chart Stress Test
//!
//! A stress-testing example for the chart widget with:
//! - FPS counter to monitor performance
//! - GUI-configurable stress parameters
//! - Toggles for Fill vs Stroke (testing Hybrid Engine optimizations)
//! - Pre-calculated geometry to isolate rendering performance
//! - Advanced styling controls (Size, Opacity, Stroke Style)
//! - **View-Aware Generation**: Shapes generate within the current pan/zoom bounds.
//! - **Length Modes**: Switch between Screen (px) and Plot (data) units for sizes and strokes.

use std::time::Instant;

use aksel::{PlotPoint, Scale, scale::Linear};
use iced::{
    Alignment, Color, Element, Point, Subscription, Task, Theme,
    mouse::ScrollDelta,
    widget::{Slider, button, checkbox, column, radio, row, text},
};
use iced_aksel::{
    Axis, Chart, DragDelta, Length, Plot, State, Stroke, StrokeStyle,
    axis::{self, Position},
    plot,
    shape::{self, Arc, Circle, Line, Polygon, Polyline, Rectangle, Triangle},
};
use rand::Rng;

const AXIS_ID_X: &str = "x";
const AXIS_ID_Y: &str = "y";

type AxisId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthMode {
    Screen,
    Plot,
}

impl std::fmt::Display for LengthMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LengthMode::Screen => write!(f, "Screen (px)"),
            LengthMode::Plot => write!(f, "Plot (units)"),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    // Counts
    RectangleCountChanged(f32),
    CircleCountChanged(f32),
    TriangleCountChanged(f32),
    LineCountChanged(f32),
    PolylineCountChanged(f32),
    ArcCountChanged(f32),
    PolygonCountChanged(f32),

    // Geometry Generation
    MinSizeChanged(f32),
    MaxSizeChanged(f32),
    OpacityChanged(f32),
    SizeModeChanged(LengthMode),

    // Shape Specifics
    // Line/Poly
    PolySegmentsChanged(f32),
    ToggleArrowStart(bool),
    ToggleArrowEnd(bool),
    ToggleInfiniteStart(bool),
    ToggleInfiniteEnd(bool),
    // Arc
    InnerRadiusChanged(f32),
    ArcSweepChanged(f32),
    // Polygon
    PolygonVerticesChanged(f32),
    TogglePolygonConcave(bool),

    // Rendering Styles
    ToggleFill(bool),
    ToggleStroke(bool),
    StrokeWidthChanged(f32),
    StrokeWidthModeChanged(LengthMode),
    StrokeStyleChanged(StrokeStyle),

    // Actions
    RegenerateAll,
    // Chart interaction
    ChartDragged(DragDelta),
    ChartScrolled(Point, ScrollDelta),
}

// --- Layers ---

struct StressRectangles {
    geometry: Vec<Rectangle<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressRectangles {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        for (base_rect, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut rect = base_rect.clone();

            if self.show_fill {
                rect = rect.fill(color);
            }

            if self.show_stroke {
                let thickness = match self.stroke_width_mode {
                    LengthMode::Screen => Length::Screen(self.stroke_width),
                    LengthMode::Plot => Length::Plot(self.stroke_width as f64),
                };
                rect =
                    rect.stroke(Stroke::new(Color::WHITE, thickness).with_style(self.stroke_style));
            }

            if self.show_fill || self.show_stroke {
                plot.add_shape(rect);
            }
        }
    }
}

struct StressCircles {
    geometry: Vec<Circle<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressCircles {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        for (base_circle, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut circle = base_circle.clone();

            if self.show_fill {
                circle = circle.fill(color);
            }

            if self.show_stroke {
                let thickness = match self.stroke_width_mode {
                    LengthMode::Screen => Length::Screen(self.stroke_width),
                    LengthMode::Plot => Length::Plot(self.stroke_width as f64),
                };
                circle = circle
                    .stroke(Stroke::new(Color::WHITE, thickness).with_style(self.stroke_style));
            }

            if self.show_fill || self.show_stroke {
                plot.add_shape(circle);
            }
        }
    }
}

struct StressTriangles {
    geometry: Vec<Triangle<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressTriangles {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        for (base_tri, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut tri = base_tri.clone();

            if self.show_fill {
                tri = tri.fill(color);
            }

            if self.show_stroke {
                let thickness = match self.stroke_width_mode {
                    LengthMode::Screen => Length::Screen(self.stroke_width),
                    LengthMode::Plot => Length::Plot(self.stroke_width as f64),
                };
                tri =
                    tri.stroke(Stroke::new(Color::WHITE, thickness).with_style(self.stroke_style));
            }

            if self.show_fill || self.show_stroke {
                plot.add_shape(tri);
            }
        }
    }
}

struct StressLines {
    geometry: Vec<Line<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    arrow_start: bool,
    arrow_end: bool,
    extend_start: bool,
    extend_end: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressLines {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        if !self.show_fill {
            return;
        }

        for (base_line, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut line = base_line.clone();

            let thickness = match self.stroke_width_mode {
                LengthMode::Screen => Length::Screen(self.stroke_width),
                LengthMode::Plot => Length::Plot(self.stroke_width as f64),
            };

            line.stroke = Stroke::new(color, thickness).with_style(self.stroke_style);
            line.arrow_start = self.arrow_start;
            line.arrow_end = self.arrow_end;
            line.extend_start = self.extend_start;
            line.extend_end = self.extend_end;

            plot.add_shape(line);
        }
    }
}

struct StressPolylines {
    geometry: Vec<Polyline<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    arrow_start: bool,
    arrow_end: bool,
    extend_start: bool,
    extend_end: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressPolylines {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        if !self.show_fill {
            return;
        }

        for (base_poly, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut poly = base_poly.clone();

            let thickness = match self.stroke_width_mode {
                LengthMode::Screen => Length::Screen(self.stroke_width),
                LengthMode::Plot => Length::Plot(self.stroke_width as f64),
            };

            poly.stroke = Stroke::new(color, thickness).with_style(self.stroke_style);
            poly.arrow_start = self.arrow_start;
            poly.arrow_end = self.arrow_end;
            poly.extend_start = self.extend_start;
            poly.extend_end = self.extend_end;

            plot.add_shape(poly);
        }
    }
}

struct StressArcs {
    geometry: Vec<Arc<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressArcs {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        for (base_arc, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut arc = base_arc.clone();

            if self.show_fill {
                arc = arc.fill(color);
            }

            if self.show_stroke {
                let thickness = match self.stroke_width_mode {
                    LengthMode::Screen => Length::Screen(self.stroke_width),
                    LengthMode::Plot => Length::Plot(self.stroke_width as f64),
                };
                arc =
                    arc.stroke(Stroke::new(Color::WHITE, thickness).with_style(self.stroke_style));
            }

            if self.show_fill || self.show_stroke {
                plot.add_shape(arc);
            }
        }
    }
}

struct StressPolygons {
    geometry: Vec<Polygon<f64>>,
    colors: Vec<Color>,
    show_fill: bool,
    show_stroke: bool,
    stroke_width: f32,
    stroke_width_mode: LengthMode,
    stroke_style: StrokeStyle,
}

impl<R: plot::Renderer> plot::Items<f64, R> for StressPolygons {
    fn draw(&self, plot: &mut Plot<'_, f64, R>, _theme: &iced::Theme) {
        for (base_poly, &color) in self.geometry.iter().zip(self.colors.iter()) {
            let mut poly = base_poly.clone();

            if self.show_fill {
                poly = poly.fill(color);
            }

            if self.show_stroke {
                let thickness = match self.stroke_width_mode {
                    LengthMode::Screen => Length::Screen(self.stroke_width),
                    LengthMode::Plot => Length::Plot(self.stroke_width as f64),
                };
                poly =
                    poly.stroke(Stroke::new(Color::WHITE, thickness).with_style(self.stroke_style));
            }

            if self.show_fill || self.show_stroke {
                plot.add_shape(poly);
            }
        }
    }
}

// --- App ---

struct StressTestApp {
    state: State<AxisId, f64>,
    rectangles_layer: StressRectangles,
    circles_layer: StressCircles,
    triangles_layer: StressTriangles,
    lines_layer: StressLines,
    polylines_layer: StressPolylines,
    arcs_layer: StressArcs,
    polygons_layer: StressPolygons,

    // Generation Configuration
    rectangle_count: usize,
    circle_count: usize,
    triangle_count: usize,
    line_count: usize,
    polyline_count: usize,
    arc_count: usize,
    polygon_count: usize,

    // Shape Specifics
    poly_segments: usize,
    polygon_vertices: usize,
    polygon_concave: bool,
    arc_inner_radius: f32,
    arc_sweep: f32,

    min_size: f32,
    max_size: f32,
    opacity: f32,
    size_mode: LengthMode,

    // FPS counter
    last_frame_time: Option<Instant>,
    fps: f32,
    frame_times: Vec<f32>,
}

impl StressTestApp {
    fn init() -> (Self, Task<Message>) {
        let mut state: State<AxisId, f64> = State::new();

        state.set_axis(
            AXIS_ID_X,
            Axis::new(Linear::new(0.0, 1000.0), Position::Bottom),
        );
        state.set_axis(
            AXIS_ID_Y,
            Axis::new(Linear::new(0.0, 1000.0), Position::Left),
        );

        let mut app = Self {
            state,
            rectangles_layer: StressRectangles {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            circles_layer: StressCircles {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            triangles_layer: StressTriangles {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            lines_layer: StressLines {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                arrow_start: false,
                arrow_end: false,
                extend_start: false,
                extend_end: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            polylines_layer: StressPolylines {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                arrow_start: false,
                arrow_end: false,
                extend_start: false,
                extend_end: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            arcs_layer: StressArcs {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            polygons_layer: StressPolygons {
                geometry: Vec::new(),
                colors: Vec::new(),
                show_fill: true,
                show_stroke: false,
                stroke_width: 2.0,
                stroke_width_mode: LengthMode::Screen,
                stroke_style: StrokeStyle::Solid,
            },
            // Start with 0 for all shapes
            rectangle_count: 0,
            circle_count: 0,
            triangle_count: 0,
            line_count: 0,
            polyline_count: 0,
            arc_count: 0,
            polygon_count: 0,

            poly_segments: 5,
            polygon_vertices: 6,
            polygon_concave: false,
            arc_inner_radius: 0.5,
            arc_sweep: 270.0,

            min_size: 10.0,
            max_size: 50.0,
            opacity: 0.8,
            size_mode: LengthMode::Screen,

            last_frame_time: None,
            fps: 0.0,
            frame_times: Vec::with_capacity(60),
        };

        // Don't generate anything at start (counts are 0)
        // app.generate_all();

        (app, Task::none())
    }

    fn get_view_bounds(&self) -> ((f64, f64), (f64, f64)) {
        let (x_min, x_max) = self
            .state
            .get_axis(&AXIS_ID_X)
            .map(|axis| {
                let (min, max) = axis.scale().domain();
                if min <= max {
                    (*min, *max)
                } else {
                    (*max, *min)
                }
            })
            .unwrap_or((0.0, 1000.0));

        let (y_min, y_max) = self
            .state
            .get_axis(&AXIS_ID_Y)
            .map(|axis| {
                let (min, max) = axis.scale().domain();
                if min <= max {
                    (*min, *max)
                } else {
                    (*max, *min)
                }
            })
            .unwrap_or((0.0, 1000.0));

        ((x_min, x_max), (y_min, y_max))
    }

    fn generate_rectangles(&mut self) {
        let ((x_min, x_max), (y_min, y_max)) = self.get_view_bounds();
        let mut rng = rand::rng();

        self.rectangles_layer.geometry.clear();
        self.rectangles_layer.colors.clear();
        self.rectangles_layer.geometry.reserve(self.rectangle_count);
        self.rectangles_layer.colors.reserve(self.rectangle_count);

        for _ in 0..self.rectangle_count {
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            let w_val =
                rng.random_range(self.min_size..self.max_size.max(self.min_size + 1.0)) as f64;
            let h_val =
                rng.random_range(self.min_size..self.max_size.max(self.min_size + 1.0)) as f64;

            let center = PlotPoint::new(x, y);
            let (width, height) = match self.size_mode {
                LengthMode::Screen => (Length::Screen(w_val as f32), Length::Screen(h_val as f32)),
                LengthMode::Plot => (Length::Plot(w_val), Length::Plot(h_val)),
            };

            self.rectangles_layer
                .geometry
                .push(Rectangle::new(center, width, height));
            self.rectangles_layer
                .colors
                .push(random_color(&mut rng, self.opacity));
        }
    }

    fn generate_circles(&mut self) {
        let ((x_min, x_max), (y_min, y_max)) = self.get_view_bounds();
        let mut rng = rand::rng();

        self.circles_layer.geometry.clear();
        self.circles_layer.colors.clear();
        self.circles_layer.geometry.reserve(self.circle_count);
        self.circles_layer.colors.reserve(self.circle_count);

        for _ in 0..self.circle_count {
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            let r_val = rng.random_range(
                (self.min_size / 2.0)..(self.max_size / 2.0).max(self.min_size / 2.0 + 0.1),
            ) as f64;

            let radius = match self.size_mode {
                LengthMode::Screen => Length::Screen(r_val as f32),
                LengthMode::Plot => Length::Plot(r_val),
            };

            self.circles_layer
                .geometry
                .push(Circle::new(PlotPoint::new(x, y), radius));
            self.circles_layer
                .colors
                .push(random_color(&mut rng, self.opacity));
        }
    }

    fn generate_triangles(&mut self) {
        let ((x_min, x_max), (y_min, y_max)) = self.get_view_bounds();
        let mut rng = rand::rng();

        self.triangles_layer.geometry.clear();
        self.triangles_layer.colors.clear();
        self.triangles_layer.geometry.reserve(self.triangle_count);
        self.triangles_layer.colors.reserve(self.triangle_count);

        for _ in 0..self.triangle_count {
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            let r_val = rng.random_range(
                (self.min_size / 2.0)..(self.max_size / 2.0).max(self.min_size / 2.0 + 0.1),
            ) as f64;

            let radius = match self.size_mode {
                LengthMode::Screen => Length::Screen(r_val as f32),
                LengthMode::Plot => Length::Plot(r_val),
            };

            self.triangles_layer
                .geometry
                .push(Triangle::equilateral(PlotPoint::new(x, y), radius));
            self.triangles_layer
                .colors
                .push(random_color(&mut rng, self.opacity));
        }
    }

    fn generate_lines(&mut self) {
        let ((x_min, x_max), (y_min, y_max)) = self.get_view_bounds();
        let mut rng = rand::rng();

        self.lines_layer.geometry.clear();
        self.lines_layer.colors.clear();
        self.lines_layer.geometry.reserve(self.line_count);
        self.lines_layer.colors.reserve(self.line_count);

        for _ in 0..self.line_count {
            let x1 = rng.random_range(x_min..x_max);
            let y1 = rng.random_range(y_min..y_max);
            let angle = rng.random_range(0.0..std::f64::consts::TAU);
            let len =
                rng.random_range(self.min_size..self.max_size.max(self.min_size + 1.0)) as f64;

            let x2 = x1 + angle.cos() * len;
            let y2 = y1 + angle.sin() * len;

            self.lines_layer.geometry.push(Line::new(
                PlotPoint::new(x1, y1),
                PlotPoint::new(x2, y2),
                Stroke::new(Color::BLACK, Length::Screen(1.0)),
            ));
            self.lines_layer
                .colors
                .push(random_color(&mut rng, self.opacity));
        }
    }

    fn generate_polylines(&mut self) {
        let ((x_min, x_max), (y_min, y_max)) = self.get_view_bounds();
        let mut rng = rand::rng();

        self.polylines_layer.geometry.clear();
        self.polylines_layer.colors.clear();
        self.polylines_layer.geometry.reserve(self.polyline_count);
        self.polylines_layer.colors.reserve(self.polyline_count);

        for _ in 0..self.polyline_count {
            let mut points = Vec::with_capacity(self.poly_segments);
            let start_x = rng.random_range(x_min..x_max);
            let start_y = rng.random_range(y_min..y_max);
            points.push(PlotPoint::new(start_x, start_y));

            let mut prev_x = start_x;
            let mut prev_y = start_y;
            let step_size = (self.max_size as f64).min(50.0);

            for _ in 1..self.poly_segments.max(2) {
                let angle = rng.random_range(0.0..std::f64::consts::TAU);
                let next_x = prev_x + angle.cos() * step_size;
                let next_y = prev_y + angle.sin() * step_size;
                points.push(PlotPoint::new(next_x, next_y));
                prev_x = next_x;
                prev_y = next_y;
            }

            self.polylines_layer.geometry.push(Polyline::new(
                points,
                Stroke::new(Color::BLACK, Length::Screen(1.0)),
            ));
            self.polylines_layer
                .colors
                .push(random_color(&mut rng, self.opacity));
        }
    }

    fn generate_arcs(&mut self) {
        let ((x_min, x_max), (y_min, y_max)) = self.get_view_bounds();
        let mut rng = rand::rng();

        self.arcs_layer.geometry.clear();
        self.arcs_layer.colors.clear();
        self.arcs_layer.geometry.reserve(self.arc_count);
        self.arcs_layer.colors.reserve(self.arc_count);

        for _ in 0..self.arc_count {
            let x = rng.random_range(x_min..x_max);
            let y = rng.random_range(y_min..y_max);
            let r_val = rng.random_range(
                (self.min_size / 2.0)..(self.max_size / 2.0).max(self.min_size / 2.0 + 0.1),
            ) as f64;

            let start_angle = rng.random_range(0.0..std::f32::consts::TAU);
            let sweep_rad = self.arc_sweep.to_radians();
            let end_angle = start_angle + sweep_rad;

            let (radius, inner_radius) = match self.size_mode {
                LengthMode::Screen => (
                    Length::Screen(r_val as f32),
                    Length::Screen(r_val as f32 * self.arc_inner_radius),
                ),
                LengthMode::Plot => (
                    Length::Plot(r_val),
                    Length::Plot(r_val * self.arc_inner_radius as f64),
                ),
            };

            let arc = Arc::new(PlotPoint::new(x, y), radius, start_angle, end_angle)
                .inner_radius(inner_radius);

            self.arcs_layer.geometry.push(arc);
            self.arcs_layer
                .colors
                .push(random_color(&mut rng, self.opacity));
        }
    }

    fn generate_polygons(&mut self) {
        let ((x_min, x_max), (y_min, y_max)) = self.get_view_bounds();
        let mut rng = rand::rng();

        self.polygons_layer.geometry.clear();
        self.polygons_layer.colors.clear();
        self.polygons_layer.geometry.reserve(self.polygon_count);
        self.polygons_layer.colors.reserve(self.polygon_count);

        for _ in 0..self.polygon_count {
            let cx = rng.random_range(x_min..x_max);
            let cy = rng.random_range(y_min..y_max);
            let radius_base = rng.random_range(
                (self.min_size / 2.0)..(self.max_size / 2.0).max(self.min_size / 2.0 + 0.1),
            ) as f64;

            let mut points = Vec::with_capacity(self.polygon_vertices);
            let step = std::f64::consts::TAU / self.polygon_vertices as f64;

            for i in 0..self.polygon_vertices {
                let theta = i as f64 * step;

                let r = if self.polygon_concave && i % 2 != 0 {
                    radius_base * 0.5
                } else {
                    radius_base
                };

                let px = cx + theta.cos() * r;
                let py = cy + theta.sin() * r;
                points.push(PlotPoint::new(px, py));
            }

            self.polygons_layer.geometry.push(Polygon::new(points));
            self.polygons_layer
                .colors
                .push(random_color(&mut rng, self.opacity));
        }
    }

    fn generate_all(&mut self) {
        self.generate_rectangles();
        self.generate_circles();
        self.generate_triangles();
        self.generate_lines();
        self.generate_polylines();
        self.generate_arcs();
        self.generate_polygons();
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                if let Some(last) = self.last_frame_time {
                    let delta = now.duration_since(last).as_secs_f32();
                    if delta > 0.0 {
                        let instant_fps = 1.0 / delta;
                        self.fps = self.fps * 0.9 + instant_fps * 0.1;
                        self.frame_times.push(delta * 1000.0);
                        if self.frame_times.len() > 60 {
                            self.frame_times.remove(0);
                        }
                    }
                }
                self.last_frame_time = Some(now);
                Task::none()
            }
            // Generation Parameters
            Message::RectangleCountChanged(v) => {
                self.rectangle_count = v as usize;
                self.generate_rectangles();
                Task::none()
            }
            Message::CircleCountChanged(v) => {
                self.circle_count = v as usize;
                self.generate_circles();
                Task::none()
            }
            Message::TriangleCountChanged(v) => {
                self.triangle_count = v as usize;
                self.generate_triangles();
                Task::none()
            }
            Message::LineCountChanged(v) => {
                self.line_count = v as usize;
                self.generate_lines();
                Task::none()
            }
            Message::PolylineCountChanged(v) => {
                self.polyline_count = v as usize;
                self.generate_polylines();
                Task::none()
            }
            Message::ArcCountChanged(v) => {
                self.arc_count = v as usize;
                self.generate_arcs();
                Task::none()
            }
            Message::PolygonCountChanged(v) => {
                self.polygon_count = v as usize;
                self.generate_polygons();
                Task::none()
            }

            Message::PolySegmentsChanged(v) => {
                self.poly_segments = v as usize;
                self.generate_polylines();
                Task::none()
            }
            Message::PolygonVerticesChanged(v) => {
                self.polygon_vertices = v as usize;
                self.generate_polygons();
                Task::none()
            }
            Message::TogglePolygonConcave(v) => {
                self.polygon_concave = v;
                self.generate_polygons();
                Task::none()
            }
            Message::InnerRadiusChanged(v) => {
                self.arc_inner_radius = v;
                self.generate_arcs();
                Task::none()
            }
            Message::ArcSweepChanged(v) => {
                self.arc_sweep = v;
                self.generate_arcs();
                Task::none()
            }

            // Global Geometry Changes -> Must Regenerate ALL
            Message::MinSizeChanged(v) => {
                self.min_size = v;
                if self.min_size > self.max_size {
                    self.max_size = self.min_size;
                }
                self.generate_all();
                Task::none()
            }
            Message::MaxSizeChanged(v) => {
                self.max_size = v;
                if self.max_size < self.min_size {
                    self.min_size = self.max_size;
                }
                self.generate_all();
                Task::none()
            }
            Message::OpacityChanged(v) => {
                self.opacity = v;
                self.generate_all();
                Task::none()
            }
            Message::SizeModeChanged(mode) => {
                self.size_mode = mode;
                self.generate_all();
                Task::none()
            }
            // Render Parameters
            Message::ToggleFill(v) => {
                self.rectangles_layer.show_fill = v;
                self.circles_layer.show_fill = v;
                self.triangles_layer.show_fill = v;
                self.lines_layer.show_fill = v;
                self.polylines_layer.show_fill = v;
                self.arcs_layer.show_fill = v;
                self.polygons_layer.show_fill = v;
                Task::none()
            }
            Message::ToggleStroke(v) => {
                self.rectangles_layer.show_stroke = v;
                self.circles_layer.show_stroke = v;
                self.triangles_layer.show_stroke = v;
                self.arcs_layer.show_stroke = v;
                self.polygons_layer.show_stroke = v;
                Task::none()
            }
            Message::StrokeWidthChanged(v) => {
                self.rectangles_layer.stroke_width = v;
                self.circles_layer.stroke_width = v;
                self.triangles_layer.stroke_width = v;
                self.lines_layer.stroke_width = v;
                self.polylines_layer.stroke_width = v;
                self.arcs_layer.stroke_width = v;
                self.polygons_layer.stroke_width = v;
                Task::none()
            }
            Message::StrokeWidthModeChanged(mode) => {
                self.rectangles_layer.stroke_width_mode = mode;
                self.circles_layer.stroke_width_mode = mode;
                self.triangles_layer.stroke_width_mode = mode;
                self.lines_layer.stroke_width_mode = mode;
                self.polylines_layer.stroke_width_mode = mode;
                self.arcs_layer.stroke_width_mode = mode;
                self.polygons_layer.stroke_width_mode = mode;
                Task::none()
            }
            Message::StrokeStyleChanged(v) => {
                self.rectangles_layer.stroke_style = v;
                self.circles_layer.stroke_style = v;
                self.triangles_layer.stroke_style = v;
                self.lines_layer.stroke_style = v;
                self.polylines_layer.stroke_style = v;
                self.arcs_layer.stroke_style = v;
                self.polygons_layer.stroke_style = v;
                Task::none()
            }
            // Line/Poly Features (Just toggles, no regen needed)
            Message::ToggleArrowStart(v) => {
                self.lines_layer.arrow_start = v;
                self.polylines_layer.arrow_start = v;
                Task::none()
            }
            Message::ToggleArrowEnd(v) => {
                self.lines_layer.arrow_end = v;
                self.polylines_layer.arrow_end = v;
                Task::none()
            }
            Message::ToggleInfiniteStart(v) => {
                self.lines_layer.extend_start = v;
                self.polylines_layer.extend_start = v;
                Task::none()
            }
            Message::ToggleInfiniteEnd(v) => {
                self.lines_layer.extend_end = v;
                self.polylines_layer.extend_end = v;
                Task::none()
            }

            Message::RegenerateAll => {
                self.generate_all();
                Task::none()
            }
            // Chart
            Message::ChartDragged(delta) => {
                let x = delta.x as f64;
                let y = delta.y as f64;
                self.state.pan_scales(AXIS_ID_X, AXIS_ID_Y, x, y);
                Task::none()
            }
            Message::ChartScrolled(point, delta) => {
                if let ScrollDelta::Lines { x: _, y } = delta {
                    let factor = 1.1f64.powf(y.into());
                    self.state.zoom_scales(
                        AXIS_ID_X,
                        AXIS_ID_Y,
                        point.x.into(),
                        point.y.into(),
                        factor,
                    );
                };
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let chart = Chart::new(&self.state)
            .layer(&self.rectangles_layer, AXIS_ID_X, AXIS_ID_Y)
            .layer(&self.circles_layer, AXIS_ID_X, AXIS_ID_Y)
            .layer(&self.triangles_layer, AXIS_ID_X, AXIS_ID_Y)
            .layer(&self.lines_layer, AXIS_ID_X, AXIS_ID_Y)
            .layer(&self.polylines_layer, AXIS_ID_X, AXIS_ID_Y)
            .layer(&self.arcs_layer, AXIS_ID_X, AXIS_ID_Y)
            .layer(&self.polygons_layer, AXIS_ID_X, AXIS_ID_Y)
            .on_drag(Message::ChartDragged)
            .on_scroll(Message::ChartScrolled);

        let avg_frame_time = if !self.frame_times.is_empty() {
            self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
        } else {
            0.0
        };

        // --- Controls Layout ---

        // 1. Stats
        let stats_row = row![
            text(format!("FPS: {:.0}", self.fps)).size(24),
            text(format!("Frame Time: {:.2}ms", avg_frame_time)).size(16),
            text(format!(
                "Objects: {}",
                self.rectangle_count
                    + self.circle_count
                    + self.triangle_count
                    + self.line_count
                    + self.polyline_count
                    + self.arc_count
                    + self.polygon_count
            ))
            .size(16),
        ]
        .spacing(20);

        // 2. Count Sliders (Column 1) - Updated to 150,000 max
        let counts_col = column![
            header("Counts"),
            slider_row(
                "Rectangles",
                self.rectangle_count as f32,
                150000.0,
                Message::RectangleCountChanged
            ),
            slider_row(
                "Circles",
                self.circle_count as f32,
                150000.0,
                Message::CircleCountChanged
            ),
            slider_row(
                "Triangles",
                self.triangle_count as f32,
                150000.0,
                Message::TriangleCountChanged
            ),
            slider_row(
                "Lines",
                self.line_count as f32,
                150000.0,
                Message::LineCountChanged
            ),
            slider_row(
                "Polylines",
                self.polyline_count as f32,
                150000.0,
                Message::PolylineCountChanged
            ),
            slider_row(
                "Arcs",
                self.arc_count as f32,
                150000.0,
                Message::ArcCountChanged
            ),
            slider_row(
                "Polygons",
                self.polygon_count as f32,
                150000.0,
                Message::PolygonCountChanged
            ),
        ]
        .spacing(5)
        .padding(5)
        .width(iced::Length::FillPortion(1));

        // 3. Geometry (Column 2)
        let geometry_col = column![
            header("Geometry"),
            checkbox_row(
                "Show Fill",
                self.rectangles_layer.show_fill,
                Message::ToggleFill
            ),
            slider_row("Min Sz", self.min_size, 200.0, Message::MinSizeChanged),
            slider_row("Max Sz", self.max_size, 200.0, Message::MaxSizeChanged),
            slider_row("Opacity", self.opacity, 1.0, Message::OpacityChanged),
            row![
                text("Mode:").size(12),
                radio(
                    "Screen",
                    LengthMode::Screen,
                    Some(self.size_mode),
                    Message::SizeModeChanged
                )
                .size(12),
                radio(
                    "Plot",
                    LengthMode::Plot,
                    Some(self.size_mode),
                    Message::SizeModeChanged
                )
                .size(12),
            ]
            .spacing(10)
        ]
        .spacing(5)
        .padding(5)
        .width(iced::Length::FillPortion(1));

        // 4. Style (Column 3)
        let style_col = column![
            header("Stroke Style"),
            checkbox_row(
                "Show Stroke",
                self.rectangles_layer.show_stroke,
                Message::ToggleStroke
            ),
            slider_row(
                "Width",
                self.rectangles_layer.stroke_width,
                20.0,
                Message::StrokeWidthChanged
            ),
            row![
                radio(
                    "Px",
                    LengthMode::Screen,
                    Some(self.rectangles_layer.stroke_width_mode),
                    Message::StrokeWidthModeChanged
                )
                .size(12),
                radio(
                    "Unit",
                    LengthMode::Plot,
                    Some(self.rectangles_layer.stroke_width_mode),
                    Message::StrokeWidthModeChanged
                )
                .size(12),
            ]
            .spacing(10),
            column![
                radio(
                    "Solid",
                    StrokeStyle::Solid,
                    Some(self.rectangles_layer.stroke_style),
                    Message::StrokeStyleChanged
                )
                .size(12),
                radio(
                    "Dashed",
                    StrokeStyle::Dashed,
                    Some(self.rectangles_layer.stroke_style),
                    Message::StrokeStyleChanged
                )
                .size(12),
                radio(
                    "Dotted",
                    StrokeStyle::Dotted,
                    Some(self.rectangles_layer.stroke_style),
                    Message::StrokeStyleChanged
                )
                .size(12),
            ]
            .spacing(2)
        ]
        .spacing(5)
        .padding(5)
        .width(iced::Length::FillPortion(1));

        // 5. Shape Specifics (Column 4)
        let shape_spec_col = column![
            header("Shape Specifics"),
            // Arc
            sub_header("Arc"),
            slider_row(
                "Inner R",
                self.arc_inner_radius,
                0.9,
                Message::InnerRadiusChanged
            ),
            slider_row("Sweep", self.arc_sweep, 360.0, Message::ArcSweepChanged),
            // Polygon
            sub_header("Polygon"),
            slider_row(
                "Verts",
                self.polygon_vertices as f32,
                20.0,
                Message::PolygonVerticesChanged
            ),
            checkbox_row(
                "Concave (Star)",
                self.polygon_concave,
                Message::TogglePolygonConcave
            ),
        ]
        .spacing(5)
        .padding(5)
        .width(iced::Length::FillPortion(1));

        // 6. Line/Poly Features (Column 5)
        let line_poly_col = column![
            header("Line/Poly Features"),
            slider_row(
                "Segs",
                self.poly_segments as f32,
                100.0,
                Message::PolySegmentsChanged
            ),
            checkbox_row(
                "Arrow Start",
                self.lines_layer.arrow_start,
                Message::ToggleArrowStart
            ),
            checkbox_row(
                "Arrow End",
                self.lines_layer.arrow_end,
                Message::ToggleArrowEnd
            ),
            checkbox_row(
                "Inf Start",
                self.lines_layer.extend_start,
                Message::ToggleInfiniteStart
            ),
            checkbox_row(
                "Inf End",
                self.lines_layer.extend_end,
                Message::ToggleInfiniteEnd
            ),
        ]
        .spacing(5)
        .padding(5)
        .width(iced::Length::FillPortion(1));

        // Combine controls
        let controls_row = row![
            counts_col,
            geometry_col,
            style_col,
            shape_spec_col,
            line_poly_col
        ]
        .spacing(10);

        let regenerate_btn = button(
            text("Regenerate All")
                .width(iced::Length::Fill)
                .align_x(Alignment::Center),
        )
        .on_press(Message::RegenerateAll)
        .padding(10)
        .width(iced::Length::Fill);

        column![stats_row, controls_row, regenerate_btn, chart]
            .spacing(10)
            .padding(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::window::frames().map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run() -> iced::Result {
        iced::application(Self::init, Self::update, Self::view)
            .theme(Self::theme)
            .subscription(Self::subscription)
            .antialiasing(true)
            .run()
    }
}

// UI Helpers

fn header(text_content: &'static str) -> Element<'static, Message> {
    text(text_content)
        .size(14)
        .style(|t: &Theme| text::Style {
            color: Some(t.palette().primary),
        })
        .into()
}

fn sub_header(text_content: &'static str) -> Element<'static, Message> {
    text(text_content)
        .size(12)
        .style(|t: &Theme| text::Style {
            color: Some(t.palette().success),
        })
        .into()
}

// Helper for compact sliders
fn slider_row(label: &str, value: f32, max: f32, msg: fn(f32) -> Message) -> Element<'_, Message> {
    // Improved Step Logic:
    // Large Counts (150k) -> 500
    // Angles (360) -> 1.0
    // Small (20) -> 0.5
    // Tiny (1.0) -> 0.05
    let step = if max > 1000.0 {
        500.0
    } else if max >= 360.0 {
        1.0
    } else if max > 5.0 {
        0.5
    } else {
        0.05
    };

    column![
        row![text(label).size(12), text(format!("{:.1}", value)).size(12)].spacing(5),
        Slider::new(0.0..=max, value, msg).step(step)
    ]
    .into()
}

// Helper for labeled checkboxes
fn checkbox_row(label: &str, value: bool, msg: fn(bool) -> Message) -> Element<'_, Message> {
    row![checkbox(value).on_toggle(msg), text(label).size(12)]
        .spacing(5)
        .align_y(Alignment::Center)
        .into()
}

fn random_color(rng: &mut impl rand::Rng, opacity: f32) -> Color {
    Color::from_rgba(
        rng.random_range(0.0..1.0),
        rng.random_range(0.0..1.0),
        rng.random_range(0.0..1.0),
        opacity,
    )
}

pub fn main() -> iced::Result {
    StressTestApp::run()
}
