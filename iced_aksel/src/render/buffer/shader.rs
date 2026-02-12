use crate::render::Primitive;
use iced_core::Rectangle;
use iced_wgpu::primitive::Renderer as PrimitiveRenderer;

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

    /// Flush shader primitives using the WGPU renderer
    /// This only works with iced_wgpu::Renderer specifically
    pub fn flush(&mut self, renderer: &mut iced_wgpu::Renderer, clip_bounds: &Rectangle) {
        if let Some(primitive) = self.mesh.take() {
            // Use the primitive::Renderer trait method
            PrimitiveRenderer::draw_primitive(renderer, *clip_bounds, primitive);
        }
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        let mesh = self.mesh.get_or_insert_with(|| mesh::AkselMesh::new());
        mesh.push_primitive(primitive);
    }
}
