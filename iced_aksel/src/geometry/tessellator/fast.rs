use crate::geometry::traits::GeometryWriter;
use iced_core::{Color, Point};
use iced_graphics::color::Packed;
use iced_graphics::mesh::{Indexed, SolidVertex2D};

/// A temporary handle that implements GeometryWriter.
/// It holds the *state* (color, pivot) that MeshData doesn't store.
pub struct FillMeshWriter<'a> {
    pub(crate) mesh: &'a mut Indexed<SolidVertex2D>,
    pub(crate) start_index: u32,
    pub(crate) packed_color: Packed,
}

impl<'a> GeometryWriter for FillMeshWriter<'a> {
    fn move_to(&mut self, p: Point) {
        // Start a new fan section. Capture the current index as our "Pivot".
        self.start_index = self.mesh.vertices.len() as u32;

        self.mesh.vertices.push(SolidVertex2D {
            position: [p.x, p.y],
            color: self.packed_color,
        });
    }

    fn line_to(&mut self, p: Point) {
        let current_index = self.mesh.vertices.len() as u32;

        // 1. Push the vertex data
        self.mesh.vertices.push(SolidVertex2D {
            position: [p.x, p.y],
            color: self.packed_color,
        });

        // 2. Analytic Triangulation (Fan Logic)
        // If we have (Pivot + Previous + Current), we form a triangle.
        if current_index > self.start_index + 1 {
            self.mesh.indices.push(self.start_index); // A (Pivot)
            self.mesh.indices.push(current_index - 1); // B (Previous)
            self.mesh.indices.push(current_index); // C (Current)
        }
    }

    fn close(&mut self) {
        // No-Op for Filled Triangle Fans.
        // GPU will do this automatically
    }
}
