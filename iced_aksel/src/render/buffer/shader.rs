use crate::render::Primitive;
use iced_core::Rectangle;

mod atlas;
mod data;
mod mesh;
mod pipeline;

pub struct ShaderBatcher {
    mesh: Option<mesh::AkselMesh>,
}

impl ShaderBatcher {
    pub fn new() -> Self {
        Self { mesh: None }
    }

    pub fn flush<R: crate::Renderer>(&mut self, renderer: &mut R, clip_bounds: &Rectangle) {
        if let Some(primitive) = self.mesh.take() {
            renderer.draw_primitive(*clip_bounds, primitive);
        }
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        let mesh = self.mesh.get_or_insert_with(|| mesh::AkselMesh::new());
        mesh.push_primitive(primitive);
    }
}
