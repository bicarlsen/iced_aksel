use super::Primitive;

use crate::stroke::{ResolvedStroke, StrokeStyle};
use iced_core::alignment::{Horizontal, Vertical};
use iced_core::text::Shaping;
use iced_core::{Point, Rectangle, Vector};
use iced_graphics::geometry::{Cache, Frame, LineCap, LineDash, LineJoin, Path, Style, Text};

const PRE_ALLOC_PATHS: usize = 5000;

pub struct PathBatcher<Renderer: crate::Renderer> {
    buffer: Vec<Primitive>,
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

    /// Renders a primitive into this path buffer.
    ///
    /// This converts the primitive into tiny-skia compatible paths.
    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.buffer.push(primitive)
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
            let primitives =
                std::mem::replace(&mut self.buffer, Vec::with_capacity(PRE_ALLOC_PATHS));
            let geometry = self
                .cache
                .draw_with_bounds(renderer, *clip_bounds, move |frame| {
                    primitives
                        .into_iter()
                        .for_each(|primitive| Self::draw_primitive(primitive, frame))
                });

            renderer.draw_geometry(geometry);
        }
    }

    fn draw_primitive(primitive: Primitive, frame: &mut Frame<Renderer>) {
        // -------------------------------------------------------------------------
        // 1. Handle Text (The "Odd One Out")
        // -------------------------------------------------------------------------
        if let Primitive::Text {
            content,
            position,
            size,
            line_height,
            bounds,
            horizontal_alignment,
            vertical_alignment,
            fill,
            font,
            rotation,
            ..
        } = primitive
        {
            frame.with_save(|frame| {
                // 1. Calculate Bounds & Clip Rec
                // We must offset the clip rectangle based on alignment so it matches the text placement.
                let (max_width, clip_rect) = if bounds.width.is_infinite() {
                    (f32::INFINITY, None)
                } else {
                    // --- THE FIX: Calculate Origin based on Alignment ---
                    let x_origin = match horizontal_alignment {
                        Horizontal::Left => position.x,
                        Horizontal::Center => position.x - (bounds.width / 2.0),
                        Horizontal::Right => position.x - bounds.width,
                    };

                    let y_origin = match vertical_alignment {
                        Vertical::Top => position.y,
                        Vertical::Center => position.y - (bounds.height / 2.0),
                        Vertical::Bottom => position.y - bounds.height,
                    };

                    (
                        bounds.width,
                        Some(Rectangle::new(Point::new(x_origin, y_origin), bounds)),
                    )
                };

                // 2. Rotation (Rotate around the generic anchor point)
                // Note: If you want to rotate the *box* around the specific alignment point,
                // this logic is correct.
                if rotation != 0.0 {
                    frame.translate(Vector::new(position.x, position.y));
                    frame.rotate(rotation);
                    frame.translate(Vector::new(-position.x, -position.y));
                }

                // 3. Draw Text
                let draw_text = |frame: &mut Frame<Renderer>| {
                    frame.fill_text(Text {
                        content: content.clone(),
                        position, // Draw at anchor
                        color: fill,
                        size,
                        // Ensure this is Absolute to prevent the "Crazy Line Height"
                        line_height,
                        font,
                        align_x: horizontal_alignment.into(),
                        align_y: vertical_alignment.into(),
                        shaping: Shaping::Advanced,
                        max_width,
                    });
                };

                // 4. Clip & Execute
                if let Some(rect) = clip_rect {
                    frame.with_clip(rect, draw_text);
                } else {
                    draw_text(frame);
                }
            });
            return;
        }

        // A. Extract Styles
        let (fill, stroke) = primitive.resolve_stroke();

        // 2. Build the Intermediate Representation
        // This runs the shared math (Bezier approx, etc.)
        let buffer = primitive.build_geometry();

        // 3. Convert IR to Iced Path
        // We assume 'buffer.populate_iced' exists as defined in previous step
        let path = Path::new(|b| buffer.populate_iced(b));

        // 4. Draw
        if let Some(color) = fill {
            frame.fill(&path, color);
        }

        if let Some(s) = stroke {
            let mut storage = [0.0; 2];
            frame.stroke(&path, create_iced_stroke(&s, &mut storage));
        }
    }
}

// --- Helper Function ---
// This prevents code duplication between Rectangle, Triangle, Circle, etc.
fn create_iced_stroke<'a>(
    s: &ResolvedStroke,
    storage: &'a mut [f32; 2],
) -> iced_graphics::geometry::Stroke<'a> {
    let (segments, line_cap) = match s.style {
        StrokeStyle::Solid => (&[] as &[f32], LineCap::Butt),
        StrokeStyle::Dashed { dash, gap } => {
            storage[0] = dash * s.thickness;
            storage[1] = gap * s.thickness;
            (&storage[..], LineCap::Butt)
        }
        StrokeStyle::Dotted { gap } => {
            storage[0] = 0.0;
            storage[1] = gap * s.thickness;
            (&storage[..], LineCap::Round)
        }
    };

    iced_graphics::geometry::Stroke {
        style: Style::Solid(s.fill),
        width: s.thickness,
        line_cap,
        line_join: LineJoin::Miter, // Miter makes sharp triangle corners look sharp
        line_dash: LineDash {
            segments,
            offset: 0,
        },
    }
}
