use crate::{
    Quality, Stroke,
    render::{MeshBuffer, buffer::PathBuffer},
};
use iced_core::{
    Color, Font, Pixels, Point, Rectangle, Size,
    alignment::{Horizontal, Vertical},
    text::{LineHeight, Wrapping},
};
use iced_graphics::mesh::SolidVertex2D;

// Describes a **shared** primitive interface between the Mesh and Path backends.
pub enum Primitive<D> {
    Rectangle {
        min: Point,
        max: Point,
        fill: Option<Color>,
        stroke: Option<(Stroke<D>, f32, f32)>,
    },
    Circle {
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
        width: f32,
        stroke: Stroke<D>,
        clip_bounds: Rectangle,
        extensions: (bool, bool),
        arrows: (bool, bool, f32),
    },
    PolyLine {
        points: Vec<Point>,
        stroke: Stroke<D>,
        width: f32,
        clip_bounds: Rectangle,
        extensions: (bool, bool),
        arrows: (bool, bool, f32),
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
        quality: Quality,
        line_height: LineHeight,
        bounds: Size,
        wrapping: Wrapping,
    },
}

pub enum Buffer {
    Path(PathBuffer),
    Mesh(MeshBuffer),
}

impl Buffer {
    pub fn flush<R: crate::plot::Renderer>(&mut self, renderer: &mut R, clip_bounds: &Rectangle) {
        match self {
            Buffer::Path(buf) => {
                buf.flush(renderer, clip_bounds);
            }
            Buffer::Mesh(buf) => {
                buf.flush(renderer, clip_bounds);
            }
        }
    }

    pub fn add_primitive<D>(&mut self, primitive: Primitive<D>) {
        match self {
            Buffer::Mesh(buf) => {
                buf.add_primitive(primitive);
            }
            Buffer::Path(buf) => {
                buf.add_primitive(primitive);
            }
        }
    }
}
