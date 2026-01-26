//! Plot rendering and data traits.
//!
//! This module provides the core plotting infrastructure for rendering data on charts.
//! The main entry point is the [`PlotData`] trait, which you implement to draw your data.

use crate::shape::Shape;

use aksel::{Float, PlotRect, Transform};
use iced_core::Font;

pub use crate::render::{Buffer, MeshBuffer, Tessellator};

/// Normalized drag delta for panning operations.
///
/// Values are in the range 0.0-1.0 and can be passed directly to axis `pan` methods.
///
/// # Example
///
/// ```rust
/// use iced_aksel::plot::DragDelta;
///
/// let delta = DragDelta { x: 0.1, y: 0.05 };
/// // Use with state.pan_axes(..., delta.x, delta.y)
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DragDelta {
    /// Normalized horizontal drag distance (0.0-1.0).
    pub x: f32,
    /// Normalized vertical drag distance (0.0-1.0).
    pub y: f32,
}

pub enum Backend {
    Mesh,
    Path,
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
    fn backend(&self) -> Backend;
}

impl Renderer for iced_renderer::fallback::Renderer<iced_wgpu::Renderer, iced_tiny_skia::Renderer> {
    fn backend(&self) -> Backend {
        match self {
            Self::Primary(_) => Backend::Mesh,
            Self::Secondary(_) => Backend::Path,
        }
    }
}

impl Renderer for iced_wgpu::Renderer {
    fn backend(&self) -> Backend {
        Backend::Mesh
    }
}

impl Renderer for iced_tiny_skia::Renderer {
    fn backend(&self) -> Backend {
        Backend::Path
    }
}

/// Trait for drawable data on a plot.
///
/// Implement this trait for your data types to render them on a chart. The `draw` method
/// receives a [`Plot`] context where you can add shapes.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{plot::{Plot, PlotData}, PlotPoint, shape::Ellipse, Measure};
/// use iced::{Color, Theme};
///
/// struct DataPoints {
///     points: Vec<PlotPoint<f64>>,
/// }
///
/// impl PlotData<f64> for DataPoints {
///     fn draw(&self, plot: &mut Plot<f64>, theme: &Theme) {
///         for point in &self.points {
///             plot.add_shape(
///                 Ellipse::new(*point, Measure::Screen(20.0), Measure::Screen(10.0))
///                     .fill(theme.palette().primary)
///             );
///         }
///     }
/// }
/// ```
pub trait PlotData<D, R = iced_renderer::Renderer, Theme = iced_core::Theme>
where
    D: Float,
    R: Renderer,
{
    /// Draws this data onto the plot.
    ///
    /// Use `plot.add_shape()` to add visual elements to the chart.
    fn draw(&self, plot: &mut Plot<D, R>, theme: &Theme);
}

/// Internal rendering context for shapes.
///
/// Manages layer ordering and buffering for efficient rendering.
pub struct Context<'a, D: Float, Renderer: self::Renderer = iced_renderer::Renderer> {
    transform: &'a Transform<'a, D, f32, f32>,
    clip_bounds: &'a iced_core::Rectangle,
    renderer: &'a mut Renderer,
    tessellators: &'a mut Tessellator,
    buffer: &'a mut Buffer,
}

impl<'a, D: Float, Renderer: self::Renderer> Context<'a, D, Renderer> {
    /// Returns the default font of the underlying renderer
    #[inline(always)]
    pub fn default_font(&mut self) -> Font {
        self.renderer.default_font()
    }

    /// Renders a shape (lines, polygons, etc.).
    ///
    /// Used internally by shapes to add geometry to the buffer.
    pub fn render<F>(&mut self, f: F)
    where
        F: FnOnce(&Transform<'a, D, f32, f32>, &mut Buffer),
    {
        // Draw mesh
        f(self.transform, self.buffer);

        // If mesh buffer and exceeds limit, render the mesh
        if let Buffer::Mesh(buffer) = self.buffer
            && buffer.vertices_count() >= buffer.limit()
        {
            buffer.flush(self.renderer, self.clip_bounds);
        }
    }
}

/// The plot rendering context for drawing shapes.
///
/// This is passed to your [`PlotData::draw`] implementation. Use [`Plot::add_shape`]
/// to render visual elements.
pub struct Plot<'a, D: Float, R: self::Renderer = iced_renderer::Renderer> {
    context: Context<'a, D, R>,
}

impl<'a, D, R> Plot<'a, D, R>
where
    D: Float,
    R: self::Renderer,
{
    /// Creates a new plot context.
    ///
    /// This is typically called internally by the Chart widget.
    pub const fn new(
        tessellators: &'a mut Tessellator,
        renderer: &'a mut R,
        clip_bounds: &'a iced_core::Rectangle,
        mesh_buffer: &'a mut MeshBuffer,
        transform: &'a Transform<'a, D, f32, f32>,
    ) -> Self {
        let context = Context {
            transform,
            clip_bounds,
            renderer,
            tessellators,
            mesh_buffer,
        };
        Self { context }
    }

    /// Returns the current plot bounds in data space.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::plot::{Plot, PlotData};
    /// # struct MyData;
    /// # impl PlotData<f64> for MyData {
    /// #     fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
    /// let bounds = plot.bounds();
    /// let (x_min, x_max) = (bounds.min_x(), bounds.max_x());
    /// let (y_min, y_max) = (bounds.min_y(), bounds.max_y());
    /// #     }
    /// # }
    /// ```
    pub fn bounds(&self) -> PlotRect<D> {
        self.context.transform.plot_bounds()
    }

    /// Adds a shape to the plot.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use iced_aksel::{plot::{Plot, PlotData}, PlotPoint, shape::Ellipse, Measure};
    /// # use iced::Color;
    /// # struct MyData;
    /// # impl PlotData<f64> for MyData {
    /// #     fn draw(&self, plot: &mut Plot<f64>, theme: &iced::Theme) {
    /// plot.add_shape(
    ///     Ellipse::new(PlotPoint::new(5.0, 10.0), Measure::Screen(20.0), Measure::Screen(10.0))
    ///         .fill(Color::from_rgb(1.0, 0.0, 0.0))
    /// );
    /// #     }
    /// # }
    /// ```
    pub fn add_shape<S: Shape<D, R>>(&mut self, shape: S) {
        shape.render(&mut self.context)
    }
}

impl<'a, D, R> Drop for Plot<'a, D, R>
where
    D: Float,
    R: self::Renderer,
{
    fn drop(&mut self) {
        self.context
            .buffer
            .flush(self.context.renderer, self.context.clip_bounds);
    }
}
