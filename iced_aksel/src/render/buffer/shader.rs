use crate::render::Primitive;
use iced_core::Rectangle;
use iced_graphics::text::cosmic_text;
use iced_wgpu::primitive::Renderer as PrimitiveRenderer;

mod atlas;
mod data;
mod mesh;
mod pipeline;

pub struct ShaderBatcher {
    buffer: Vec<Primitive>,
    cache: mesh::ShaderCache,
    text_buffer: cosmic_text::Buffer,
    cache_updated: bool, // Track if cache has been updated for current buffer
}

impl ShaderBatcher {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            cache: mesh::ShaderCache::new(),
            // Initialize a text-buffer
            //
            // Proper metrics need to be set when drawing the text
            text_buffer: cosmic_text::Buffer::new_empty(cosmic_text::Metrics::new(1.0, 1.0)),
            cache_updated: false,
        }
    }

    /// Clear the buffer, triggering a redraw
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cache_updated = false; // Mark that cache needs updating when buffer is refilled
    }

    /// Check if the buffer is empty (Should redraw)
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn draw(&mut self, renderer: &mut impl PrimitiveRenderer, clip_bounds: &Rectangle) {
        // Only update cache when buffer was just filled (not already cached)
        if !self.is_empty() && !self.cache_updated {
            let primitives = self.buffer.clone();
            self.cache.update(primitives.into());
            self.cache_updated = true;
        }

        renderer.draw_primitive(*clip_bounds, self.cache.clone());
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.buffer.push(primitive);
    }
}
