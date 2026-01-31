use super::{DrawGeometry, GeometryWriter};
use iced_core::Point;

/// A rectangle defined by two opposite corners.
pub struct RectangleGeometry {
    top_left: Point,
    bottom_right: Point,
}

impl RectangleGeometry {
    /// Creates a new rectangle geometry from any two corners.
    /// Automatically sorts them to ensure Top-Left / Bottom-Right order.
    pub fn new(p1: Point, p2: Point) -> Self {
        Self {
            top_left: Point::new(p1.x.min(p2.x), p1.y.min(p2.y)),
            bottom_right: Point::new(p1.x.max(p2.x), p1.y.max(p2.y)),
        }
    }
}

impl DrawGeometry for RectangleGeometry {
    fn draw<W: GeometryWriter>(&self, writer: &mut W) {
        let x0 = self.top_left.x;
        let y0 = self.top_left.y;
        let x1 = self.bottom_right.x;
        let y1 = self.bottom_right.y;

        // Winding Order: Top-Left -> Top-Right -> Bottom-Right -> Bottom-Left
        // This specific order works perfectly for both "Fan" triangulation and Path stroking.
        writer.move_to(Point::new(x0, y0));
        writer.line_to(Point::new(x1, y0));
        writer.line_to(Point::new(x1, y1));
        writer.line_to(Point::new(x0, y1));
        writer.close();
    }
}
