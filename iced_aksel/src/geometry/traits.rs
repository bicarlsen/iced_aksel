use crate::geometry::target::MeshTarget;
use crate::stroke::ResolvedStroke;
use iced_core::{Color, Point};

pub trait GeometricShape {
    fn draw<W: GeometryWriter>(&self, builder: &mut W);
}

pub trait GeometryWriter {
    fn move_to(&mut self, p: Point);
    fn line_to(&mut self, p: Point);
    fn close(&mut self);
}
