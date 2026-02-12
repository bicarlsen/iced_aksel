use crate::{Quality, render::primitive::Primitive, stroke::StrokeStyle};

use iced_core::{Rectangle, Transformation};
use iced_graphics::mesh::{Cache, Indexed, Mesh, Renderer, SolidVertex2D};

mod tessellation;

use tessellation::{Tessellator, manual::linear};

const PRE_ALLOC_VERTICES: usize = 10_000;
const PRE_ALLOC_INDICES: usize = 20_000;

/// Raw mesh data storage.
///
/// This struct holds the actual vertex and index buffers, separated from the
/// tessellation logic. This allows the tessellator to operate on the data
/// without borrowing conflicts.
#[derive(Default)]
pub struct MeshData {
    /// The actual data awaiting upload to the GPU.
    buffer: Option<Indexed<SolidVertex2D>>,
}

impl MeshData {
    /// Appends raw geometry (indices and vertices) to the buffer.
    ///
    /// This is used by the `ManualTessellator` (for circles/rects) to push data directly.
    pub fn add(&mut self, indices: &[u32], vertices: &[SolidVertex2D]) {
        let mesh = self.get_mesh_mut();
        let start_offset = mesh.vertices.len() as u32;
        mesh.vertices.extend_from_slice(vertices);
        // We must offset the new indices because they refer to the start of *their*
        // list, but we are appending them to the *end* of our global list.
        mesh.indices
            .extend(indices.iter().map(|i| i + start_offset));
    }

    /// Provides mutable access to the underlying mesh structure.
    ///
    /// Used by `LyonAdapter` and `TextTessellator` to write directly into the vectors.
    /// Uses lazy initialization to avoid allocating memory if the buffer is unused.
    pub(crate) fn get_mesh_mut(&mut self) -> &mut Indexed<SolidVertex2D> {
        self.buffer.get_or_insert_with(|| Indexed {
            // Pre-allocate a reasonable chunk of memory (10k vertices) to reduce re-allocations
            vertices: Vec::with_capacity(PRE_ALLOC_VERTICES),
            indices: Vec::with_capacity(PRE_ALLOC_INDICES),
        })
    }

    pub(crate) fn total_vertices(&self) -> usize {
        if let Some(buf) = &self.buffer {
            return buf.vertices.len();
        };
        0
    }

    pub(crate) fn to_mesh(self, clip_bounds: Rectangle) -> Option<Mesh> {
        self.buffer.map(|buffers| Mesh::Solid {
            buffers,
            transformation: Transformation::IDENTITY,
            clip_bounds,
        })
    }
}

/// A simplified container for GPU vertex and index data.
///
/// It manages the lifecycle of `iced_graphics::mesh::Indexed`, automatically
/// flushing it to the renderer when it exceeds its capacity or when the frame ends.
pub struct MeshBatcher {
    /// The raw mesh data storage.
    pub(crate) buffer: Vec<Primitive>,
    pub(crate) cached: Option<Cache>,

    /// The mesh-tessellation cache/builder.
    pub(crate) tessellator: Tessellator,

    /// A soft limit for vertices per batch.
    /// If exceeded, the `Context` (in `plot.rs`) will trigger a flush.
    vertex_limit: usize,
}

impl MeshBatcher {
    /// Creates a new buffer with a specific soft limit.
    pub fn new(vertex_limit: usize) -> Self {
        Self {
            buffer: Vec::new(),
            cached: None,
            tessellator: Tessellator::new(),
            vertex_limit,
        }
    }

    /// Sets the quality of the internal renderer
    pub fn set_quality(&mut self, quality: Quality) {
        self.tessellator.set_quality(quality);
    }

    /// Returns the soft limit configured for this buffer.
    pub const fn limit(&self) -> usize {
        self.vertex_limit
    }

    /// Flushes the pending geometry to the `iced` renderer.
    ///
    /// This consumes the current internal buffer and resets it.
    pub(crate) fn flush<R>(&mut self, renderer: &mut R, clip_bounds: &Rectangle, with_damage: bool)
    where
        R: Renderer,
    {
        if with_damage {
            self.cached = None;
        }

        if !self.buffer.is_empty() && self.cached.is_none() {
            let primitives = std::mem::replace(&mut self.buffer, Vec::new());
            let mut meshes = Vec::new();
            let mut current_buffer = MeshData::default();
            primitives.into_iter().for_each(|primitive| {
                Self::draw_primitive(&mut current_buffer, primitive, &mut self.tessellator);
                if current_buffer.total_vertices() > self.vertex_limit {
                    let buf = std::mem::replace(&mut current_buffer, MeshData::default());
                    if let Some(mesh) = buf.to_mesh(*clip_bounds) {
                        meshes.push(mesh);
                    }
                }
            });

            self.cached = Some(Cache::new(meshes.into()));
        }

        if let Some(cached) = self.cached.clone() {
            renderer.draw_mesh_cache(cached);
        }
    }

    pub fn add_primitive(&mut self, primitive: Primitive) {
        self.buffer.push(primitive);
    }

    /// Renders a primitive into this mesh buffer using the tessellator.
    fn draw_primitive(buffer: &mut MeshData, primitive: Primitive, tessellator: &mut Tessellator) {
        match primitive {
            Primitive::Rectangle {
                xy1: min,
                xy2: max,
                fill,
                stroke,
            } => {
                tessellator.draw_rectangle(buffer, min.x, min.y, max.x, max.y, fill, stroke);
            }
            Primitive::Ellipse {
                center,
                radius,
                fill,
                stroke,
            } => {
                tessellator.draw_ellipse(buffer, center, radius, fill, stroke);
            }
            Primitive::Triangle {
                points,
                fill,
                stroke,
            } => {
                tessellator.draw_triangle(buffer, points[0], points[1], points[2], fill, stroke);
            }
            Primitive::Polygon {
                center,
                radius,
                vertices,
                rotation,
                fill,
                stroke,
            } => {
                tessellator.draw_polygon(buffer, center, radius, vertices, rotation, fill, stroke);
            }
            Primitive::Line {
                start,
                end,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                tessellator.draw_line(buffer, start, end, stroke, clip_bounds, extensions, arrows);
            }
            Primitive::HorizontalLine {
                y,
                x_start,
                x_end,
                stroke,
                snap,
            } => match stroke.style {
                StrokeStyle::Solid => {
                    linear::draw_horizontal_line(
                        buffer,
                        x_start,
                        x_end,
                        y,
                        stroke.thickness,
                        stroke.fill,
                        snap,
                    );
                }
                StrokeStyle::Dashed { dash, gap } => linear::draw_horizontal_dashed_line(
                    buffer,
                    x_start,
                    x_end,
                    y,
                    stroke.thickness,
                    stroke.fill,
                    dash,
                    gap,
                    snap,
                ),
                StrokeStyle::Dotted { gap: _ } => todo!("Draw dotted line"),
            },
            Primitive::VerticalLine {
                x,
                y_start,
                y_end,
                stroke,
                snap,
            } => match stroke.style {
                StrokeStyle::Solid => {
                    linear::draw_vertical_line(
                        buffer,
                        x,
                        y_start,
                        y_end,
                        stroke.thickness,
                        stroke.fill,
                        snap,
                    );
                }
                StrokeStyle::Dashed { dash, gap } => linear::draw_vertical_dashed_line(
                    buffer,
                    x,
                    y_start,
                    y_end,
                    stroke.thickness,
                    stroke.fill,
                    dash,
                    gap,
                    snap,
                ),
                StrokeStyle::Dotted { gap: _ } => todo!("Draw dotted line"),
            },
            Primitive::PolyLine {
                points,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                tessellator.draw_polyline(buffer, points, stroke, clip_bounds, extensions, arrows);
            }
            Primitive::BezierCurve {
                start,
                end,
                control_1,
                control_2,
                stroke,
            } => {
                tessellator.draw_bezier(buffer, start, control_1, control_2, end, stroke);
            }
            Primitive::Spline {
                points,
                stroke,
                tension,
            } => {
                tessellator.draw_spline(buffer, points, stroke, tension);
            }
            Primitive::Arc {
                center,
                radius_inner,
                radius_outer,
                start_angle,
                end_angle,
                fill,
                stroke,
            } => {
                tessellator.draw_arc(
                    buffer,
                    center.x,
                    center.y,
                    radius_inner,
                    radius_outer,
                    start_angle,
                    end_angle,
                    fill,
                    stroke,
                );
            }
            Primitive::Area {
                points,
                fill,
                stroke,
            } => {
                tessellator.draw_area(buffer, &points, fill, stroke);
            }
            Primitive::Text {
                font,
                content,
                position,
                size,
                rotation,
                horizontal_alignment,
                vertical_alignment,
                fill,
                quality,
                line_height,
                bounds,
                wrapping,
            } => {
                let tolerance = quality
                    .map(|q| q.max(0.001))
                    .unwrap_or_else(|| tessellator.text_tolerance());

                tessellator.draw_text(
                    buffer,
                    crate::render::text::Text {
                        font,
                        content,
                        position,
                        size,
                        rotation,
                        horizontal_alignment,
                        vertical_alignment,
                        fill,
                        tolerance,
                        line_height: line_height.to_absolute(size),
                        bounds,
                        wrapping,
                    },
                );
            }
        }
    }
}
