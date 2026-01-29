use super::Primitive;

use iced_core::{Point, Rectangle, Size};
use iced_graphics::geometry::{Cache, Fill, Path};

const PRE_ALLOC_PATHS: usize = 5000;

pub struct PathBatcher<Renderer: crate::Renderer> {
    buffer: Vec<(Path, Fill)>,
    cache: Cache<Renderer>,
    paths_limit: usize,
}

impl<Renderer: crate::render::Renderer> PathBatcher<Renderer> {
    pub fn new(paths_limit: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(PRE_ALLOC_PATHS),
            cache: Cache::new(),
            paths_limit,
        }
    }

    pub const fn paths_count(&self) -> usize {
        self.buffer.len()
    }

    pub const fn limit(&self) -> usize {
        self.paths_limit
    }

    pub(crate) fn flush(
        &mut self,
        renderer: &mut Renderer,
        clip_bounds: &Rectangle,
        with_damage: bool,
    ) {
        if with_damage {
            self.cache.clear();
        }

        if !self.buffer.is_empty() {
            let paths = std::mem::replace(&mut self.buffer, Vec::with_capacity(PRE_ALLOC_PATHS));
            let geometry = self
                .cache
                .draw_with_bounds(renderer, *clip_bounds, move |frame| {
                    paths
                        .into_iter()
                        .for_each(|(path, fill)| frame.fill(&path, fill));
                });

            renderer.draw_geometry(geometry);
        }
    }

    pub fn add(&mut self, path: Path, fill: Fill) {
        self.buffer.push((path, fill));
    }

    /// Renders a primitive into this path buffer.
    ///
    /// This converts the primitive into tiny-skia compatible paths.
    pub fn add_primitive(&mut self, primitive: Primitive) {
        match primitive {
            Primitive::Rectangle {
                xy1,
                xy2,
                fill,
                stroke,
            } => {
                let left_most = xy1.x.min(xy2.x);
                let right_most = xy1.x.max(xy2.x);

                let top_most = xy1.y.min(xy2.y);
                let bottom_most = xy1.y.max(xy2.y);

                let size = Size {
                    width: left_most - right_most,
                    height: bottom_most - top_most,
                };

                let top_left = Point::new(left_most, top_most);

                let path = Path::rectangle(top_left, size);

                if let Some(color) = fill {
                    self.add(path, color.into())
                }
            }
            _ => {}
        }

        // todo!("Implement path rendering for tiny-skia backend")
    }
}
