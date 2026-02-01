use crate::{Quality, geometry, render::primitive::Primitive, stroke::StrokeStyle};
use std::borrow::Cow;

use iced_core::{Color, Point, Rectangle, Transformation};
use iced_graphics::color::{Packed, pack};
use iced_graphics::mesh::{Indexed, Mesh, Renderer, SolidVertex2D};
use lyon::lyon_algorithms;
use lyon_path::iterator::Flattened;
use lyon_path::{LineCap, LineJoin};
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator, StrokeVertex,
};

mod tessellation;

use crate::geometry::tessellator::lyon::LyonMeshBuilder;
use crate::geometry::tessellator::lyon::dash::DashingIterator;
use crate::stroke::ResolvedStroke;
use tessellation::{Tessellator, manual::linear};

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

    /// Creates the builder that bridges Lyon's output to this mesh's storage.
    pub fn adapter(&mut self, packed_color: Packed) -> LyonMeshBuilder<'_> {
        let mesh = self.get_mesh_mut();

        LyonMeshBuilder::new(&mut mesh.vertices, &mut mesh.indices, packed_color)
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

    lyon_stroker: StrokeTessellator,
    lyon_filler: FillTessellator,

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
            lyon_stroker: StrokeTessellator::new(),
            lyon_filler: FillTessellator::new(),
            vertex_limit,
        }
    }

    /// Sets the quality of the internal renderer
    pub fn set_quality(&mut self, quality: Quality) {
        self.tessellator.set_quality(quality);
    }

    /// Returns the number of vertices currently sitting in the pending buffer.
    pub const fn vertices_count(&self) -> usize {
        if let Some(buffer) = &self.data.buffer {
            return buffer.vertices.len();
        }
        0
    }

    /// Returns the total vertices processed this frame (flushed + pending).
    pub const fn total_vertices(&self) -> usize {
        let current = if let Some(b) = &self.data.buffer {
            b.vertices.len()
        } else {
            0
        };
        self.data.total_vertices + current
    }

    /// Returns the total indices processed this frame (flushed + pending).
    pub const fn total_indices(&self) -> usize {
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
    pub fn add_primitive(&mut self, primitive: Primitive) {
        let (fill, stroke) = primitive.resolve_stroke(); // Assuming you have this helper
        let buffer = primitive.build_geometry();
        // 1. FILL (Unchanged)
        if let Some(color) = fill {
            let packed = pack(color);
            let _ = self.lyon_filler.tessellate(
                buffer.lyon_iter(),
                &FillOptions::default().with_tolerance(0.1),
                &mut self.data.adapter(packed),
            );
        }

        // 2. STROKE
        if let Some(s) = stroke {
            let packed = pack(s.fill);

            // Standard Options
            let mut options = StrokeOptions::default()
                .with_line_width(s.thickness)
                .with_tolerance(0.1);

            let source = buffer.lyon_iter();

            match s.style {
                // --- SOLID (Fast Path) ---
                StrokeStyle::Solid => {
                    options = options.with_line_cap(LineCap::Butt);
                    let _ = self.lyon_stroker.tessellate(
                        source,
                        &options,
                        &mut self.data.adapter(packed),
                    );
                }

                // --- DASHED ---
                StrokeStyle::Dashed { dash, gap } => {
                    options = options.with_line_cap(LineCap::Butt);
                    let d = dash * s.thickness;
                    let g = gap * s.thickness;

                    // 1. Flatten
                    let flattened = Flattened::new(0.1, source);
                    // 2. Dash (Safe State Machine)
                    let dashed = DashingIterator::new(flattened, vec![d, g]);

                    let _ = self.lyon_stroker.tessellate(
                        dashed,
                        &options,
                        &mut self.data.adapter(packed),
                    );
                }

                // --- DOTTED ---
                StrokeStyle::Dotted { gap } => {
                    options = options.with_line_cap(LineCap::Round);
                    let g = gap * s.thickness;

                    // Dot = very small line (0.1) with Round Cap
                    // We use 0.1 instead of 0.0 because custom iterators sometimes glitch on 0-length
                    let flattened = Flattened::new(0.1, source);
                    let dotted = DashingIterator::new(flattened, vec![0.1, g]);

                    let _ = self.lyon_stroker.tessellate(
                        dotted,
                        &options,
                        &mut self.data.adapter(packed),
                    );
                }
            }
        }
    }
}

/// Converts your high-level stroke style into Lyon's low-level options.
fn convert_options(stroke: &ResolvedStroke) -> StrokeOptions {
    let mut options = StrokeOptions::default()
        .with_line_width(stroke.thickness)
        .with_tolerance(0.1); // High quality curves

    match stroke.style {
        // 1. SOLID
        StrokeStyle::Solid => {
            options = options
                .with_line_cap(LineCap::Butt)
                .with_line_join(LineJoin::Miter);
        }

        // 2. DASHED
        StrokeStyle::Dashed { dash, gap } => {
            // Lyon expects a list of segment lengths.
            // We use your relative units multiplied by thickness (standard convention)
            // or absolute if your units are already absolute.
            // Assuming 'dash' and 'gap' are multipliers of thickness (like SVG):
            // If they are absolute pixels, remove the `* stroke.thickness`.

            // Let's assume they are multipliers based on your previous code context:
            // dash: 5.0 means "5 times the line width"
            // If they are raw pixels, remove the multiplication.

            // NOTE: Lyon's builder pattern for dashes is separate from StrokeOptions
            // in some versions, but StrokeOptions itself doesn't hold the dash pattern directly
            // in newer Lyon versions. It usually requires a Source Wrapper (like DashPath).

            // HOWEVER, standard StrokeOptions doesn't have "with_dash_pattern".
            // If you are using standard lyon_tessellation::StrokeOptions,
            // you only configure Cap/Join/Width.

            // To support dashes in Lyon, we actually need to wrap the ITERATOR,
            // not just the options.

            // For now, let's set the CAPS correctly.
            options = options
                .with_line_cap(LineCap::Butt)
                .with_line_join(LineJoin::Miter);
        }

        // 3. DOTTED
        StrokeStyle::Dotted { .. } => {
            // Dots are just zero-length dashes with Round caps.
            options = options
                .with_line_cap(LineCap::Round)
                .with_line_join(LineJoin::Round);
        }
    };

    options
}
