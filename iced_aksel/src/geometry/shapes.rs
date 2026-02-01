use crate::geometry::traits::{GeometricShape, GeometryWriter};
use iced_core::Point;

pub struct Rectangle {
    pub xy1: Point,
    pub xy2: Point,
}

impl Rectangle {
    pub fn new(xy1: Point, xy2: Point) -> Self {
        Self { xy1, xy2 }
    }
}

impl GeometricShape for Rectangle {
    fn draw<W: GeometryWriter>(&self, writer: &mut W) {
        // 1. Sort coordinates to find the 4 corners
        let left = self.xy1.x.min(self.xy2.x);
        let right = self.xy1.x.max(self.xy2.x);
        let top = self.xy1.y.min(self.xy2.y);
        let bottom = self.xy1.y.max(self.xy2.y);

        // 2. Stream the path (Top-Left -> Top-Right -> Bottom-Right -> Bottom-Left)
        writer.move_to(Point::new(left, top)); // P0 (Pivot)
        writer.line_to(Point::new(right, top)); // P1
        writer.line_to(Point::new(right, bottom)); // P2 (Forms Triangle 1: P0-P1-P2)
        writer.line_to(Point::new(left, bottom)); // P3 (Forms Triangle 2: P0-P2-P3)
        writer.close();
    }
}
