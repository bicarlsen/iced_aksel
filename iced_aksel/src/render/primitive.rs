use crate::stroke::ResolvedStroke;
use iced_core::{
    Color, Font, Pixels, Point, Rectangle, Size,
    alignment::{Horizontal, Vertical},
    text::{self, LineHeight, Shaping, Wrapping},
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
pub enum Primitive {
    Rectangle {
        xy1: Point,
        xy2: Point,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Ellipse {
        center: Point,
        radius: Point,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Triangle {
        points: [Point; 3],
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Polygon {
        center: Point,
        radius: f32,
        vertices: u16,
        rotation: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Line {
        start: Point,
        end: Point,
        stroke: ResolvedStroke,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    },
    HorizontalLine {
        y: f32,
        x_start: f32,
        x_end: f32,
        stroke: ResolvedStroke,
        snap: bool,
    },
    VerticalLine {
        x: f32,
        y_start: f32,
        y_end: f32,
        stroke: ResolvedStroke,
        snap: bool,
    },
    PolyLine {
        points: Vec<Point>,
        stroke: ResolvedStroke,
        clip_bounds: Rectangle,
        extensions: LineExtensions,
        arrows: LineArrows,
    },
    BezierCurve {
        start: Point,
        end: Point,
        control_1: Point,
        control_2: Option<Point>,
        stroke: ResolvedStroke,
    },
    Spline {
        points: Vec<Point>,
        stroke: ResolvedStroke,
        tension: f32,
    },
    Arc {
        center: Point,
        radius_inner: f32,
        radius_outer: f32,
        start_angle: f32,
        end_angle: f32,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Area {
        points: Vec<Point>,
        fill: Option<Color>,
        stroke: Option<ResolvedStroke>,
    },
    Text {
        position: Point,
        content: String,
        bounds: Size,
        size: Pixels,
        line_height: LineHeight,
        font: Font,
        horizontal_alignment: text::Alignment,
        vertical_alignment: Vertical,
        fill: Color,
        shaping: Shaping,
        wrapping: Wrapping,
    },
}
