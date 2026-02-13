//! The central geometry collector for the rendering pipeline.
//!
//! # Role in Architecture
//! The `MeshBuffer` acts as a "Funnel". It accepts triangles from various sources:
//! * The Text Engine (Glyphs)
//! * The Shape Engine (Circles, Rectangles)
//! * The Line Engine (Polylines)
//!
//! It bundles all these tiny pieces of geometry into massive batches. This is critical for performance
//! because talking to the GPU is expensive. Sending 10,000 triangles in one call is much faster
//! than making 10,000 calls of 1 triangle each.

use super::{Primitive, Quality};
use iced_core::Rectangle;
use std::any::Any;

mod mesh;
mod path;
mod shader;

pub use mesh::{MeshBatcher, MeshData};
pub use path::PathBatcher;
pub use shader::ShaderBatcher;

pub enum RenderBuffer<Renderer: crate::Renderer> {
    Mesh(Box<MeshBatcher>),
    Path(Box<PathBatcher<Renderer>>),
    Shader(Box<ShaderBatcher>),
}

impl<Renderer: crate::Renderer> RenderBuffer<Renderer> {
    pub fn new_mesh() -> Self {
        Self::Mesh(Box::new(MeshBatcher::new()))
    }

    pub fn new_path(limit: usize) -> Self {
        Self::Path(Box::new(PathBatcher::new(limit)))
    }

    pub fn new_shader() -> Self {
        Self::Shader(Box::new(ShaderBatcher::new()))
    }

    pub fn flush(&mut self, renderer: &mut Renderer, clip_bounds: &Rectangle, with_damage: bool)
    where
        Renderer: 'static, // Required for downcasting
    {
        match self {
            Self::Path(buf) => {
                buf.flush(renderer, clip_bounds, with_damage);
            }
            Self::Mesh(buf) => {
                buf.flush(renderer, clip_bounds, with_damage);
            }
            Self::Shader(buf) => {
                // Downcast to the concrete WGPU renderer type
                //
                // We need to do this as not every renderer implements the
                // iced_wgpu::primitive::Renderer trait.
                let any_renderer = renderer as &mut dyn Any;
                if let Some(renderer) = any_renderer.downcast_mut::<iced_renderer::Renderer>() {
                    buf.flush(renderer, clip_bounds, with_damage);
                } else {
                    // This should never happen if buffer creation logic is correct
                    panic!("Shader backend was selected for a non-wgpu compatible backend!");
                }
            }
        }
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        match self {
            Self::Mesh(buf) => {
                buf.add_primitive(primitive);
            }
            Self::Path(buf) => {
                buf.add_primitive(primitive);
            }
            Self::Shader(buf) => {
                buf.add_primitive(primitive);
            }
        }
    }

    pub fn set_quality(&mut self, quality: Quality) {
        match self {
            Self::Mesh(buf) => {
                buf.tessellator.set_quality(quality);
            }
            Self::Path(_buf) => {
                // todo!("Set quality on path-buffer")
            }
            Self::Shader(_buf) => {
                // todo!("Set quality on shader-buffer")
            }
        }
    }
}
