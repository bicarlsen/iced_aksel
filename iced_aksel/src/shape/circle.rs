use crate::{
    Length, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellators},
};
use aksel::{Float, PlotPoint, Transform};
use iced::Color;
use lyon::geom::Arc;
use lyon::math::{Angle, Point, Vector};

/// A circle shape defined by a center and a radius.
#[derive(Debug, Clone)]
pub struct Circle<D> {
    pub center: PlotPoint<D>,
    pub radius: Length<D>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Circle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Circle<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    /// Creates a new Circle.
    ///
    /// # Arguments
    /// * `center` - Center position in Plot coordinates.
    /// * `radius` - Radius in either Screen pixels or Plot units.
    pub const fn new(center: PlotPoint<D>, radius: Length<D>) -> Self {
        Self {
            center,
            radius,
            fill: None,
            stroke: None,
        }
    }

    // =========================================================================
    //  Builder Methods
    // =========================================================================

    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    #[inline]
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    // =========================================================================
    //  Tessellation Logic
    // =========================================================================

    fn tessellate(
        self,
        transform: &Transform<D, D, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        // 1. Resolve Geometry
        let cx = transform.x_to_screen(&self.center.x);
        let cy = transform.y_to_screen(&self.center.y);
        let center = Point::new(cx, cy);

        // Calculate Radius in Pixels
        let r = match self.radius {
            Length::Screen(pixels) => pixels,
            Length::Plot(units) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&units);
                (p1 - p0).abs()
            }
        };

        // Cull tiny circles
        if r < 0.5 {
            return;
        }

        // 2. Resolve Stroke Thickness
        let maybe_stroke_data = if let Some(stroke) = &self.stroke {
            let width = match stroke.thickness {
                Length::Screen(w) => w,
                Length::Plot(w) => {
                    let p0 = transform.x_to_screen(&D::zero());
                    let p1 = transform.x_to_screen(&w);
                    (p1 - p0).abs()
                }
            };
            Some((width, stroke))
        } else {
            None
        };

        // 3. Rule 2: Geometric Stability (Consumption Check)
        let is_consumed = if let Some((width, _)) = maybe_stroke_data {
            width >= r
        } else {
            false
        };

        if is_consumed {
            if let Some((_, stroke)) = maybe_stroke_data {
                let arc = Arc {
                    center,
                    radii: Vector::new(r, r),
                    start_angle: Angle::radians(0.0),
                    sweep_angle: Angle::radians(std::f32::consts::TAU),
                    x_rotation: Angle::radians(0.0),
                };
                // FIX: Use .flattened() to get points, and pass to fill_polygon
                tess.fill_polygon(buffer, arc.flattened(0.1), stroke.fill);
            }
            return;
        }

        // 4. Render Fill
        if let Some(color) = self.fill {
            let fill_r = if maybe_stroke_data.is_some() {
                (r - 0.5).max(0.0)
            } else {
                r
            };

            if fill_r > 0.1 {
                let arc = Arc {
                    center,
                    radii: Vector::new(fill_r, fill_r),
                    start_angle: Angle::radians(0.0),
                    sweep_angle: Angle::radians(std::f32::consts::TAU),
                    x_rotation: Angle::radians(0.0),
                };
                // FIX: Use .flattened() to get points, and pass to fill_polygon
                tess.fill_polygon(buffer, arc.flattened(0.1), color);
            }
        }

        // 5. Render Stroke
        if let Some((width, stroke)) = maybe_stroke_data {
            let stroke_radius = r - (width / 2.0);

            if stroke_radius > 0.1 {
                let arc = Arc {
                    center,
                    radii: Vector::new(stroke_radius, stroke_radius),
                    start_angle: Angle::radians(0.0),
                    sweep_angle: Angle::radians(std::f32::consts::TAU),
                    x_rotation: Angle::radians(0.0),
                };

                // FIX: Use .flattened() to get points, and pass to stroke_polyline.
                // We pass 'true' to close the path.
                tess.stroke_polyline(buffer, arc.flattened(0.1), stroke, width, true);
            }
        }
    }
}
