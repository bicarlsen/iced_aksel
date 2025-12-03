use super::plot;

use aksel::Float;

// mod ellipse;
mod label;
// mod linesegment;
mod circle;
mod line;
mod polygon;
mod polyline;
mod rectangle;
mod sector;
mod triangle;

// pub use ellipse::{Ellipse, StrokeBehavior};
pub use label::Label;
// pub use linesegment::LineSegment;
pub use circle::Circle;
pub use line::Line;
pub use polygon::Polygon;
pub use polyline::Polyline;
pub use rectangle::Rectangle;
pub use sector::Sector;
pub use triangle::Triangle;

/// A trait for unifying rendering across "primitives"
pub trait Shape<D, Renderer = iced::Renderer>
where
    D: Float,
    Renderer: plot::Renderer,
{
    fn render(self, ctx: &mut plot::Context<'_, D, Renderer>);
}
