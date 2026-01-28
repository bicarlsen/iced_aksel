use super::Primitive;

use iced_core::Rectangle;
use iced_graphics::geometry::{Fill, Frame, Path};

const PRE_ALLOC_PATHS: usize = 5000;

pub struct PathBatcher {
    paths: Option<Vec<(Path, Fill)>>,
    paths_limit: usize,
}

impl PathBatcher {
    pub const fn new(paths_limit: usize) -> Self {
        Self {
            paths: None,
            paths_limit,
        }
    }

    pub const fn paths_count(&self) -> usize {
        if let Some(buffer) = &self.paths {
            return buffer.len();
        };
        0
    }

    pub const fn limit(&self) -> usize {
        self.paths_limit
    }

    pub(crate) fn flush<R>(&mut self, renderer: &mut R, clip_bounds: &Rectangle)
    where
        R: iced_graphics::geometry::Renderer,
    {
        if let Some(paths) = self.paths.take() {
            if paths.is_empty() {
                return;
            }

            // TODO: This might be a bit of a performance hog - Maybe there is a better way?
            let mut frame = Frame::with_bounds(renderer, *clip_bounds);
            paths
                .into_iter()
                .for_each(|(path, fill)| frame.fill(&path, fill));

            renderer.draw_geometry(frame.into_geometry());
        }
    }

    pub fn add(&mut self, path: Path, fill: Fill) {
        let paths = self.get_paths_mut();
        paths.push((path, fill));
    }

    pub fn get_paths_mut(&mut self) -> &mut Vec<(Path, Fill)> {
        self.paths
            .get_or_insert_with(|| Vec::with_capacity(PRE_ALLOC_PATHS))
    }

    /// Renders a primitive into this path buffer.
    ///
    /// This converts the primitive into tiny-skia compatible paths.
    pub fn add_primitive(&mut self, primitive: Primitive) {
        // TODO: Implement path rendering for each primitive type
        // For now, this is a placeholder
        let _ = primitive;
        todo!("Implement path rendering for tiny-skia backend")
    }
}
