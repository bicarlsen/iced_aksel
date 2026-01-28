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
use aksel::Float;
use iced_core::Rectangle;

mod mesh;
mod path;

pub use mesh::{MeshBatcher, MeshData};
pub use path::PathBatcher;

pub enum RenderBuffer {
    Mesh(Box<MeshBatcher>),
    Path(Box<PathBatcher>),
}

impl RenderBuffer {
    pub fn new_mesh(limit: usize) -> Self {
        Self::Mesh(Box::new(MeshBatcher::new(limit)))
    }

    pub fn new_path(limit: usize) -> Self {
        Self::Path(Box::new(PathBatcher::new(limit)))
    }

    pub fn flush<R: crate::Renderer>(&mut self, renderer: &mut R, clip_bounds: &Rectangle) {
        match self {
            Self::Path(buf) => {
                buf.flush(renderer, clip_bounds);
            }
            Self::Mesh(buf) => {
                buf.flush(renderer, clip_bounds);
            }
        }
    }

    pub fn add_primitive<D: Float>(&mut self, primitive: Primitive<D>) {
        match self {
            Self::Mesh(buf) => {
                buf.add_primitive(primitive);
            }
            Self::Path(buf) => {
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
                todo!("Set quality on path-buffer")
            }
        }
    }
}
