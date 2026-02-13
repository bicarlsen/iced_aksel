mod buffer;
mod primitive;
mod text;

use std::any::Any;

pub use buffer::RenderBuffer;
pub use primitive::{LineArrows, LineExtensions, Primitive};
pub use text::Text;

/// The rendering quality of a buffer.
///
/// This controls the error tolerance of the tessellation algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Quality {
    /// High triangle count, very smooth curves.
    High,
    #[default]
    /// Balanced performance and visual fidelity.
    Medium,
    /// Low triangle count, "blocky" curves. Best for performance.
    Low,
    /// Custom value.
    Custom {
        // Higher is better (High = 2.0)
        //
        // Must be between 0.1..5.0
        tessellation: f32,
        // Lower is better (High = 0.2)
        //
        // If value < 0.001, it will default to 0.001
        text: f32,
    },
}

impl Quality {
    /// Converts the quality setting into a tessellation tolerance value.
    /// Lower values mean higher precision (more triangles).
    pub(crate) const fn to_text_tolerance(self) -> f32 {
        match self {
            Self::High => 0.2,
            Self::Medium => 0.5,
            Self::Low => 1.5,
            Self::Custom { text, .. } => text.max(0.001),
        }
    }

    pub(crate) const fn to_tessellation_quality(self) -> f32 {
        match self {
            Self::High => 2.0,
            Self::Medium => 1.0,
            Self::Low => 0.5,
            Self::Custom { tessellation, .. } => tessellation.clamp(0.1, 5.0),
        }
    }
}

pub enum Backend {
    Mesh,
    Path,
    Shader,
}

/// Renderer requirements for plotting.
///
/// This trait is automatically implemented for any renderer that satisfies the requirements.
pub trait Renderer<Font = iced_core::Font>:
    iced_core::Renderer
    + iced_core::text::Renderer<Font = Font>
    + iced_graphics::geometry::Renderer
    + iced_graphics::mesh::Renderer
    + 'static
{
    fn preffered_backend(&self) -> Backend;
}

impl<A, B> Renderer<A::Font> for iced_renderer::fallback::Renderer<A, B>
where
    A: Renderer,
    B: Renderer<A::Font, Paragraph = A::Paragraph, Editor = A::Editor>,
{
    fn preffered_backend(&self) -> Backend {
        match self {
            Self::Primary(primary) => primary.preffered_backend(),
            Self::Secondary(secondary) => secondary.preffered_backend(),
        }
    }
}

// Implementation for WGPU renderer
impl Renderer for iced_wgpu::Renderer {
    fn preffered_backend(&self) -> Backend {
        Backend::Shader
    }
}

// Implementation for tiny_skia renderer
impl Renderer for iced_tiny_skia::Renderer {
    fn preffered_backend(&self) -> Backend {
        Backend::Path
    }
}
