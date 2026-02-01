use crate::geometry::target::MeshTarget;
use crate::geometry::traits::GeometricShape;

use iced_core::Color;
use iced_graphics::color::{Packed, pack};
use iced_graphics::mesh::SolidVertex2D;

use crate::stroke::ResolvedStroke;
use lyon_path::Path;
use lyon_tessellation::{
    FillGeometryBuilder, FillOptions, FillTessellator, FillVertex, StrokeGeometryBuilder,
    StrokeOptions, StrokeVertex, VertexId,
};
// --- Public API ---

/// robustly fills a shape using Lyon's tessellator.
/// This handles complex polygons, self-intersections, etc.
pub fn fill_shape<S: GeometricShape + ?Sized>(target: &mut MeshTarget, shape: &S, color: Color) {
    // 1. Build the Path
    let mut builder = Path::builder();
    shape.draw(&mut builder);
    let path = builder.build();

    // 2. Configure Adapter
    let packed_color = pack(color);
    let mut adapter = LyonAdapter::new(target.vertices, target.indices, packed_color);

    // 3. Tessellate
    // We create a temporary FillTessellator here.
    // (Optimization note: In the future, add 'filler' to MeshTarget to reuse this)
    let mut tessellator = FillTessellator::new();
    let options = FillOptions::default().with_tolerance(0.1);

    let _ = tessellator.tessellate_path(&path, &options, &mut adapter);
}

/// robustly strokes a shape using Lyon.
/// Handles thickness, dashing, and line joins.
pub fn stroke_shape<S: GeometricShape + ?Sized>(
    target: &mut MeshTarget,
    shape: &S,
    stroke: &ResolvedStroke,
) {
    // 1. Build Base Path
    let mut builder = Path::builder();
    shape.draw(&mut builder);
    let base_path = builder.build();

    // 2. Handle Dashing
    // Uses your existing dash logic.
    // If solid, we use Cow::Borrowed to avoid allocation (simulated here for clarity).
    let path = match stroke.style {
        crate::stroke::StrokeStyle::Solid => std::borrow::Cow::Borrowed(&base_path),
        crate::stroke::StrokeStyle::Dashed { dash, gap } => {
            let dashed = create_dashed_path(&base_path, &[dash, gap], 0.0);
            std::borrow::Cow::Owned(dashed)
        }
        crate::stroke::StrokeStyle::Dotted { gap } => {
            // Dot optimization: tiny dash + round cap
            let dashed = create_dashed_path(&base_path, &[0.01, gap], 0.0);
            std::borrow::Cow::Owned(dashed)
        }
    };

    // 3. Options
    let mut options = StrokeOptions::default()
        .with_line_width(stroke.thickness)
        .with_tolerance(0.1);

    if matches!(stroke.style, crate::stroke::StrokeStyle::Dotted { .. }) {
        options = options.with_line_cap(lyon_tessellation::LineCap::Round);
    }

    // 4. Tessellate
    let packed_color = pack(stroke.fill);
    let mut adapter = LyonAdapter::new(target.vertices, target.indices, packed_color);

    // Use the shared stroker from Target!
    let _ = target
        .stroker
        .tessellate_path(&path, &options, &mut adapter);
}

// --- Internal Adapter ---

/// Adapts Lyon's output to Iced's Mesh buffers.
struct LyonAdapter<'a> {
    vertices: &'a mut Vec<SolidVertex2D>,
    indices: &'a mut Vec<u32>,
    packed_color: Packed,
    start_offset: u32,
}

impl<'a> LyonAdapter<'a> {
    fn new(
        vertices: &'a mut Vec<SolidVertex2D>,
        indices: &'a mut Vec<u32>,
        packed_color: Packed,
    ) -> Self {
        Self {
            start_offset: vertices.len() as u32,
            vertices,
            indices,
            packed_color,
        }
    }
}

impl<'a> lyon_tessellation::GeometryBuilder for LyonAdapter<'a> {
    fn begin_geometry(&mut self) {
        self.start_offset = self.vertices.len() as u32;
    }
    fn end_geometry(&mut self) {}
    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        let offset = self.start_offset;
        self.indices.push(a.0 + offset);
        self.indices.push(b.0 + offset);
        self.indices.push(c.0 + offset);
    }
    fn abort_geometry(&mut self) {}
}

impl<'a> StrokeGeometryBuilder for LyonAdapter<'a> {
    fn add_stroke_vertex(
        &mut self,
        vertex: StrokeVertex,
    ) -> Result<VertexId, lyon_tessellation::GeometryBuilderError> {
        let idx = self.vertices.len() as u32;
        self.vertices.push(SolidVertex2D {
            position: vertex.position().to_array(),
            color: self.packed_color,
        });
        Ok(VertexId(idx - self.start_offset))
    }
}

impl<'a> FillGeometryBuilder for LyonAdapter<'a> {
    fn add_fill_vertex(
        &mut self,
        vertex: FillVertex,
    ) -> Result<VertexId, lyon_tessellation::GeometryBuilderError> {
        let idx = self.vertices.len() as u32;
        self.vertices.push(SolidVertex2D {
            position: vertex.position().to_array(),
            color: self.packed_color,
        });
        Ok(VertexId(idx - self.start_offset))
    }
}
