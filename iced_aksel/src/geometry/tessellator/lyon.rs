pub mod dash;

use iced_graphics::color::Packed;
use iced_graphics::mesh::SolidVertex2D;
use lyon_tessellation::{
    FillGeometryBuilder, FillVertex, GeometryBuilder, GeometryBuilderError, StrokeGeometryBuilder,
    StrokeVertex, VertexId,
};

/// The Output Sink.
/// Catches vertices from Lyon and writes them directly into the MeshData vectors.
pub struct LyonMeshBuilder<'a> {
    vertices: &'a mut Vec<SolidVertex2D>,
    indices: &'a mut Vec<u32>,
    packed_color: Packed,
    start_offset: u32,
}

impl<'a> LyonMeshBuilder<'a> {
    pub fn new(
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

// --- Implementation of Lyon Traits ---

impl<'a> GeometryBuilder for LyonMeshBuilder<'a> {
    fn begin_geometry(&mut self) {
        // Snapshot the current length so new indices are relative to this batch
        self.start_offset = self.vertices.len() as u32;
    }

    fn end_geometry(&mut self) {}

    fn add_triangle(&mut self, a: VertexId, b: VertexId, c: VertexId) {
        let offset = self.start_offset;
        // Adjust Lyon's local indices (0, 1, 2) to global buffer indices (1000, 1001, 1002)
        self.indices.push(a.0 + offset);
        self.indices.push(b.0 + offset);
        self.indices.push(c.0 + offset);
    }

    fn abort_geometry(&mut self) {
        // Optional: truncate vectors back to start_offset on failure
    }
}

impl<'a> FillGeometryBuilder for LyonMeshBuilder<'a> {
    fn add_fill_vertex(&mut self, vertex: FillVertex) -> Result<VertexId, GeometryBuilderError> {
        let idx = self.vertices.len() as u32;
        self.vertices.push(SolidVertex2D {
            position: vertex.position().to_array(),
            color: self.packed_color,
        });
        Ok(VertexId(idx - self.start_offset))
    }
}

impl<'a> StrokeGeometryBuilder for LyonMeshBuilder<'a> {
    fn add_stroke_vertex(
        &mut self,
        vertex: StrokeVertex,
    ) -> Result<VertexId, GeometryBuilderError> {
        let idx = self.vertices.len() as u32;
        self.vertices.push(SolidVertex2D {
            position: vertex.position().to_array(),
            color: self.packed_color,
        });
        Ok(VertexId(idx - self.start_offset))
    }
}
