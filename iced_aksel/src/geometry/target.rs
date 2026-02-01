use iced_graphics::mesh::SolidVertex2D;
use lyon_tessellation::StrokeTessellator;

/// A target specifically for generating GPU meshes.
pub struct MeshTarget<'a> {
    pub vertices: &'a mut Vec<SolidVertex2D>,
    pub indices: &'a mut Vec<u32>,
    pub stroker: &'a mut StrokeTessellator,
}

// Future-proofing:
// pub struct PathTarget<'a> { ... }
