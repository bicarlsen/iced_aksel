use crate::geometry::GeometrySink;

use iced_core::Point;
use iced_graphics::geometry::path::Builder;

/// Adapts the GeometrySink interface to Iced's Canvas Path Builder.
pub struct IcedGeometryBuilder<'a> {
    builder: &'a mut Builder,
}

impl<'a> IcedGeometryBuilder<'a> {
    pub fn new(builder: &'a mut Builder) -> Self {
        Self { builder }
    }
}

impl<'a> GeometrySink for IcedGeometryBuilder<'a> {
    fn move_to(&mut self, p: Point) {
        self.builder.move_to(p);
    }
    fn line_to(&mut self, p: Point) {
        self.builder.line_to(p);
    }
    fn close(&mut self) {
        self.builder.close();
    }

    fn quadratic_bezier_to(&mut self, c: Point, to: Point) {
        self.builder.quadratic_curve_to(c, to);
    }
    fn cubic_bezier_to(&mut self, c1: Point, c2: Point, to: Point) {
        self.builder.bezier_curve_to(c1, c2, to);
    }
}
