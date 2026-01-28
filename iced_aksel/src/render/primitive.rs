use crate::Stroke;
use aksel::Float;
use iced_core::{
    Color, Font, Pixels, Point, Rectangle, Size,
    alignment::{Horizontal, Vertical},
    text::{LineHeight, Wrapping},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineExtensions {
    pub start: bool,
    pub end: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineArrows {
    pub start: bool,
    pub end: bool,
    pub size: f32,
}

// Describes a **shared** primitive interface between the Mesh and Path backends.
pub enum Primitive<D: Float> {
    Rectangle {
        min: Point,
        max: Point,
        fill: Option<Color>,
        stroke: Option<(Stroke<D>, f32, f32)>,
    },
    Ellipse {
        center: Point,
        radius: Point,
        fill: Option<Color>,
        stroke: Option<(Stroke<D>, f32)>,
    },
    Triangle {
        points: [Point; 3],
        fill: Option<Color>,
        stroke: Option<(Stroke<D>, f32)>,
    },
    Polygon {
        center: Point,
        radius: f32,
        vertices: u16,
        rotation: f32,
        fill: Option<Color>,
        stroke: Option<(Stroke<D>, f32)>,
    },
    Line {
        start: Point,
        end: Point,
        stroke: Stroke<D>,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    },
    HorizontalLine {
        y: f32,
        x_start: f32,
        x_end: f32,
        stroke: Stroke<D>,
        snap: bool,
    },
    VerticalLine {
        x: f32,
        y_start: f32,
        y_end: f32,
        stroke: Stroke<D>,
        snap: bool,
    },
    PolyLine {
        points: Vec<Point>,
        stroke: Stroke<D>,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    },
    BezierCurve {
        start: Point,
        end: Point,
        control_1: Point,
        control_2: Option<Point>,
        stroke: Stroke<D>,
        width: f32,
    },
    Spline {
        points: Vec<Point>,
        stroke: Stroke<D>,
        width: f32,
        tension: f32,
    },
    Arc {
        center: Point,
        radius_inner: f32,
        radius_outer: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Option<Color>,
        stroke: Option<(Stroke<D>, f32)>,
    },
    Area {
        points: Vec<Point>,
        fill: Option<Color>,
        stroke: Option<(Stroke<D>, f32)>,
    },
    Text {
        font: Font,
        content: String,
        position: Point,
        size: Pixels,
        rotation: f32,
        horizontal_alignment: Horizontal,
        vertical_alignment: Vertical,
        fill: Color,
        /// Override the quality tolerance of the text
        quality: Option<f32>,
        line_height: LineHeight,
        bounds: Size,
        wrapping: Wrapping,
    },
}
