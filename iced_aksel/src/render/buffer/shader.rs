use crate::render::Primitive;
use iced_core::Rectangle;
use iced_graphics::text::cosmic_text;
use iced_wgpu::primitive::Renderer as PrimitiveRenderer;

mod atlas;
mod data;
mod mesh;
mod pipeline;

pub struct ShaderBatcher {
    cache: mesh::AkselMesh,
    buffer: Vec<Primitive>,
    text_buffer: cosmic_text::Buffer,
}

impl ShaderBatcher {
    pub fn new() -> Self {
        Self {
            cache: mesh::AkselMesh::new(),
            // Initialize a text-buffer
            //
            // Proper metrics need to be set when drawing the text
            text_buffer: cosmic_text::Buffer::new_empty(cosmic_text::Metrics::new(1.0, 1.0)),
        }
    }

    /// Flush shader primitives using the WGPU renderer
    /// This only works with iced_wgpu::Renderer specifically
    pub fn flush(
        &mut self,
        renderer: &mut impl PrimitiveRenderer,
        clip_bounds: &Rectangle,
        with_damage: bool,
    ) {
        if with_damage {
            self.mesh.clear();
        }

        renderer.draw_primitive(*clip_bounds, self.mesh.clone());
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        let mesh = self.mesh.get_or_insert_with(mesh::AkselMesh::new);
        mesh.push_primitive(primitive, &mut self.text_buffer);
    }
}
