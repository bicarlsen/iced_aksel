pub mod shapes;

use iced_core::Point;
pub use shapes::RectangleGeometry;

/// An interface for anything that can receive geometry commands.
///
/// This trait allows us to write shape logic once (in `geometry/shapes.rs`)
/// and use it for both Mesh generation (WebGPU) and Path building (Tiny-Skia).
pub trait GeometryWriter {
    /// Move the "pen" to a specific point without drawing.
    fn move_to(&mut self, point: Point);

    /// Draw a straight line from the current position to this point.
    fn line_to(&mut self, point: Point);

    /// Close the current sub-path, connecting back to the start.
    fn close(&mut self);
}

/// A shape that knows how to draw itself using a writer.
pub trait DrawGeometry {
    /// Stream the geometry of this shape to the provided writer.
    fn draw<W: GeometryWriter>(&self, writer: &mut W);
}
