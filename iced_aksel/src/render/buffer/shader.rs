use crate::render::Primitive;
use iced_core::Rectangle;

mod pipeline;

pub struct ShaderBatcher {}

impl ShaderBatcher {
    pub fn flush<R: crate::Renderer>(&mut self, renderer: &mut R, clip_bounds: &Rectangle) {
        todo!()
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        todo!()
    }
}
