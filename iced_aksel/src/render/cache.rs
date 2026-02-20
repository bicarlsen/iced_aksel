//! The cache for the rendering pipeline.

use super::{Primitive, Quality};
use iced_core::Rectangle;

mod math;
mod mesh;
mod path;

pub use mesh::{MeshCache, MeshData};
pub use path::PathCache;

pub enum RenderCache<Renderer: crate::Renderer> {
    Mesh(Box<MeshCache>),
    Path(Box<PathCache<Renderer>>),
}

impl<Renderer: crate::Renderer> RenderCache<Renderer> {
    pub fn new_mesh() -> Self {
        Self::Mesh(Box::new(MeshCache::new()))
    }

    pub fn new_path() -> Self {
        Self::Path(Box::new(PathCache::new()))
    }

    pub fn request_redraw(&mut self) {
        match self {
            Self::Path(cache) => {
                cache.request_redraw();
            }
            Self::Mesh(cache) => {
                cache.request_redraw();
            }
        }
    }

    pub fn needs_redraw(&self) -> bool {
        match self {
            Self::Mesh(cache) => cache.needs_redraw(),
            Self::Path(cache) => cache.needs_redraw(),
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer, clip_bounds: &Rectangle) {
        match self {
            Self::Path(cache) => {
                cache.draw(renderer, clip_bounds);
            }
            Self::Mesh(cache) => {
                cache.draw(renderer, clip_bounds);
            }
        }
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        match self {
            Self::Mesh(cache) => {
                cache.add_primitive(primitive);
            }
            Self::Path(cache) => {
                cache.add_primitive(primitive);
            }
        }
    }

    pub fn set_quality(&mut self, quality: Quality) {
        match self {
            Self::Mesh(cache) => {
                cache.set_quality(quality);
            }
            Self::Path(_cache) => {
                // todo!("Set quality on path-buffer")
            }
        }
    }
}
