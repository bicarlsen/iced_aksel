use crate::render::primitive::Primitive;

use aksel::Float;
use iced_core::{Rectangle, Transformation};
use iced_graphics::mesh::{Indexed, Mesh, Renderer, SolidVertex2D};

mod tessellation;

use tessellation::Tessellator;

const PRE_ALLOC_VERTICES: usize = 10_000;
const PRE_ALLOC_INDICES: usize = 20_000;

/// Raw mesh data storage.
///
/// This struct holds the actual vertex and index buffers, separated from the
/// tessellation logic. This allows the tessellator to operate on the data
/// without borrowing conflicts.
pub struct MeshData {
    /// The actual data awaiting upload to the GPU.
    buffer: Option<Indexed<SolidVertex2D>>,

    // Statistics for the debug overlay
    total_vertices: usize,
    total_indices: usize,
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
}

/// A simplified container for GPU vertex and index data.
///
/// It manages the lifecycle of `iced_graphics::mesh::Indexed`, automatically
/// flushing it to the renderer when it exceeds its capacity or when the frame ends.
pub struct MeshBatcher {
    /// The raw mesh data storage.
    pub(crate) data: MeshData,

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
            data: MeshData {
                buffer: None,
                total_vertices: 0,
                total_indices: 0,
            },
            tessellator: Tessellator::new(),
            vertex_limit,
        }
    }

    /// Sets the quality of the internal renderer
    pub fn set_quality(&mut self, quality: f32) {
        self.tessellator.set_quality(quality);
    }

    /// Returns the number of vertices currently sitting in the pending buffer.
    pub fn vertices_count(&self) -> usize {
        if let Some(buffer) = &self.data.buffer {
            return buffer.vertices.len();
        }
        0
    }

    /// Returns the total vertices processed this frame (flushed + pending).
    pub fn total_vertices(&self) -> usize {
        let current = if let Some(b) = &self.data.buffer {
            b.vertices.len()
        } else {
            0
        };
        self.data.total_vertices + current
    }

    /// Returns the total indices processed this frame (flushed + pending).
    pub fn total_indices(&self) -> usize {
        let current = if let Some(b) = &self.data.buffer {
            b.indices.len()
        } else {
            0
        };
        self.data.total_indices + current
    }

    /// Returns the soft limit configured for this buffer.
    pub const fn limit(&self) -> usize {
        self.vertex_limit
    }

    /// Flushes the pending geometry to the `iced` renderer.
    ///
    /// This consumes the current internal buffer and resets it.
    pub(crate) fn flush<R>(&mut self, renderer: &mut R, clip_bounds: &Rectangle)
    where
        R: Renderer,
    {
        // We `take()` the buffer, effectively clearing it from `self`.
        if let Some(buffer) = self.data.buffer.take() {
            if buffer.indices.is_empty() {
                return;
            }

            let v_count = buffer.vertices.len();
            let i_count = buffer.indices.len();

            self.data.total_vertices += v_count;
            self.data.total_indices += i_count;

            renderer.draw_mesh(Mesh::Solid {
                buffers: buffer,
                transformation: Transformation::IDENTITY,
                clip_bounds: *clip_bounds,
            });
        }
    }

    /// Renders a primitive into this mesh buffer using the tessellator.
    pub fn add_primitive<D: Float>(&mut self, primitive: Primitive<D>) {
        match primitive {
            Primitive::Rectangle {
                min,
                max,
                fill,
                stroke,
            } => {
                self.tessellator.draw_rectangle(
                    &mut self.data,
                    min.x,
                    min.y,
                    max.x,
                    max.y,
                    fill,
                    stroke,
                );
            }
            Primitive::Ellipse {
                center,
                radius,
                fill,
                stroke,
            } => {
                self.tessellator.draw_ellipse(
                    &mut self.data,
                    center.x,
                    center.y,
                    radius.x,
                    radius.y,
                    fill,
                    stroke,
                );
            }
            Primitive::Triangle {
                points,
                fill,
                stroke,
            } => {
                self.tessellator.draw_triangle(
                    &mut self.data,
                    points[0],
                    points[1],
                    points[2],
                    fill,
                    stroke,
                );
            }
            Primitive::Polygon {
                center,
                radius,
                vertices,
                rotation,
                fill,
                stroke,
            } => {
                self.tessellator.draw_polygon(
                    &mut self.data,
                    center,
                    radius,
                    vertices,
                    rotation,
                    fill,
                    stroke,
                );
            }
            Primitive::Line {
                start,
                end,
                width,
                stroke,
                clip_bounds,
                extensions,
                arrows,
            } => {
                self.tessellator.draw_line(
                    &mut self.data,
                    start,
                    end,
                    stroke,
                    width,
                    clip_bounds,
                    extensions,
                    arrows,
                );
            }
            Primitive::PolyLine {
                points,
                stroke,
                width,
                clip_bounds,
                extensions,
                arrows,
            } => {
                self.tessellator.draw_polyline(
                    &mut self.data,
                    points,
                    stroke,
                    width,
                    clip_bounds,
                    extensions,
                    arrows,
                );
            }
            Primitive::BezierCurve {
                start,
                end,
                control_1,
                control_2,
                stroke,
                width,
            } => {
                self.tessellator.draw_bezier(
                    &mut self.data,
                    start,
                    control_1,
                    control_2,
                    end,
                    stroke,
                    width,
                );
            }
            Primitive::Spline {
                points,
                stroke,
                width,
                tension,
            } => {
                self.tessellator
                    .draw_spline(&mut self.data, points, stroke, width, tension);
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
                self.tessellator.draw_arc(
                    &mut self.data,
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
                self.tessellator
                    .draw_area(&mut self.data, &points, fill, stroke);
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
                self.tessellator.draw_text(
                    &mut self.data,
                    crate::render::text::Text {
                        font,
                        content,
                        position,
                        size,
                        rotation,
                        horizontal_alignment,
                        vertical_alignment,
                        fill,
                        quality,
                        line_height: line_height.to_absolute(size),
                        bounds,
                        wrapping,
                    },
                );
            }
        }
    }
}
