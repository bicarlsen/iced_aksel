use crate::{Measure, Shape, Stroke, plot, render::Primitive};

use aksel::{Float, PlotPoint};
use iced_core::{Color, Point};

/// A primitive representing a sector of a circle or a ring.
///
/// # Usage
/// ```rust
/// use iced_aksel::shape::Arc;
/// use iced_aksel::Measure;
/// use aksel::PlotPoint;
/// use iced_core::Color;
///
/// let sector = Arc::new(
///     PlotPoint::new(0.0, 0.0),
///     Measure::Screen(50.0),
///     0.0, // Radians
///     1.5  // Radians
/// )
/// .fill(Color::from_rgb(1.0, 0.0, 0.0));
/// ```
#[derive(Debug, Clone)]
pub struct Arc<D> {
    /// The center point of the arc
    pub center: PlotPoint<D>,
    /// The outer radius of the arc
    pub radius: Measure<D>,
    /// The inner radius of the arc (0 for a pie slice)
    pub inner_radius: Measure<D>,
    /// The starting angle in radians
    pub start_angle: f32,
    /// The ending angle in radians
    pub end_angle: f32,
    /// The fill color for the arc interior
    pub fill: Option<Color>,
    /// The stroke style for the arc border
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: crate::Renderer> Shape<D, R> for Arc<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        let Self {
            center,
            radius,
            inner_radius,
            start_angle,
            end_angle,
            fill,
            stroke,
        } = self;

        let center = Point::new(ctx.x_to_screen(&center.x), ctx.y_to_screen(&center.y));

        // Calculate isotropic radii by taking the minimum scale of X and Y dimensions
        let radius_outer = {
            let x = radius.resolve_x(ctx);
            let y = radius.resolve_y(ctx);
            x.min(y)
        };

        let radius_inner = {
            let x = inner_radius.resolve_x(ctx);
            let y = inner_radius.resolve_y(ctx);
            x.min(y)
        };

        let stroke = stroke.and_then(|stroke| {
            let width_x = stroke.thickness.resolve_x(ctx);
            let width_y = stroke.thickness.resolve_y(ctx);
            let width_pixels = width_x.min(width_y);

            if width_pixels < 0.1 {
                None
            } else {
                Some((stroke, width_pixels))
            }
        });

        ctx.add_primitive(Primitive::Arc {
            center,
            radius_inner,
            radius_outer,
            start_angle,
            end_angle,
            stroke,
            fill,
        });
    }
}

impl<D: Float> Arc<D> {
    /// Creates a new `Arc`.
    ///
    /// * `radius`: The outer radius of the arc.
    /// * `start_angle`: Starting angle in **Radians**.
    /// * `end_angle`: Ending angle in **Radians**.
    ///
    /// Note: The shape is invisible by default. You must call `.fill()` or `.stroke()` to render it.
    pub const fn new(
        center: PlotPoint<D>,
        radius: Measure<D>,
        start_angle: f32,
        end_angle: f32,
    ) -> Self {
        Self {
            center,
            radius,
            inner_radius: Measure::Screen(0.0),
            start_angle,
            end_angle,
            fill: None,
            stroke: None,
        }
    }

    /// Sets the inner radius of the arc, creating a donut sector.
    pub const fn inner_radius(mut self, radius: Measure<D>) -> Self {
        self.inner_radius = radius;
        self
    }

    /// Sets the fill color of the arc.
    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke style (border) of the arc.
    #[inline]
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }
}
