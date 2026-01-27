mod buffer;
mod primitive;
mod text;

pub use buffer::{MeshBatcher, MeshData, PathBatcher, RenderBuffer};
pub use primitive::Primitive;
pub use text::Text;

/// The rendering quality of a buffer.
///
/// This controls the error tolerance of the tessellation algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Quality {
    /// High triangle count, very smooth curves. (Tolerance: 0.2)
    High,
    #[default]
    /// Balanced performance and visual fidelity. (Tolerance: 0.5)
    Medium,
    /// Low triangle count, "blocky" curves. Best for performance. (Tolerance: 1.5)
    Low,
    /// Custom tolerance value. Lower is better/slower.
    Custom(f32),
}

impl Quality {
    /// Converts the quality setting into a tessellation tolerance value.
    /// Lower values mean higher precision (more triangles).
    pub const fn to_tolerance(self) -> f32 {
        match self {
            Self::High => 0.2,
            Self::Medium => 0.5,
            Self::Low => 1.5,
            Self::Custom(val) => val.max(0.001),
        }
    }
}

/// Renderer requirements for plotting.
///
/// This trait is automatically implemented for any renderer that satisfies the requirements.
pub trait Renderer:
    iced_core::Renderer
    + iced_core::text::Renderer<Font = iced_core::Font>
    + iced_graphics::geometry::Renderer
    + iced_graphics::mesh::Renderer
{
    fn init_buffer(&self) -> RenderBuffer;
}

impl Renderer for iced_renderer::fallback::Renderer<iced_wgpu::Renderer, iced_tiny_skia::Renderer> {
    fn init_buffer(&self) -> RenderBuffer {
        match self {
            Self::Primary(primary) => primary.init_buffer(),
            Self::Secondary(secondary) => secondary.init_buffer(),
        }
    }
}

impl Renderer for iced_wgpu::Renderer {
    fn init_buffer(&self) -> RenderBuffer {
        RenderBuffer::Mesh(MeshBatcher::new(100_000))
    }
}

impl Renderer for iced_tiny_skia::Renderer {
    fn init_buffer(&self) -> RenderBuffer {
        RenderBuffer::Path(PathBatcher::new(5000)) // TODO: Test limits
    }
}
