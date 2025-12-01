use aksel::{Float, PlotPoint, Transform};
use iced::Color;
use lyon::math::point;
use lyon_path::Path;

use crate::{
    Length, Shape, Stroke, plot,
    render::{MeshBuffer, Tessellators},
};

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
            self.lyon_tessellation(transform, buffer, tess)
        })
    }
}

impl<D: Float> Circle<D> {
    pub const fn new(center: PlotPoint<D>, radius: Length<D>) -> Self {
        Self {
            center,
            radius,
            fill: None,
            stroke: None,
        }
    }

    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    fn lyon_tessellation(
        self,
        transform: &Transform<D, D, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        let cx = transform.x_to_screen(&self.center.x);
        let cy = transform.y_to_screen(&self.center.y);

        let r = match self.radius {
            Length::Screen(pixels) => pixels,
            Length::Plot(units) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&units);
                (p1 - p0).abs()
            }
        };

        if r < 0.5 {
            return;
        }

        let stroke_width = self
            .stroke
            .as_ref()
            .map_or(0.0, |stroke| match stroke.thickness {
                Length::Screen(w) => w,
                Length::Plot(w) => {
                    let p0 = transform.x_to_screen(&D::zero());
                    let p1 = transform.x_to_screen(&w);
                    (p1 - p0).abs()
                }
            });

        // SAFETY MEASURE: Massive Strokes
        // If the stroke is wider than the radius (width >= r), the "Inner hole" is mathematically closed.
        // If we try to stroke it normally, we might get negative path radii or artifacts.
        // In this case, the visual result should simply be a solid circle of the stroke color.
        if stroke_width >= r
            && let Some(stroke) = &self.stroke
        {
            let mut builder = Path::builder();
            builder.add_circle(point(cx, cy), r, lyon::path::Winding::Positive);
            tess.fill_path(buffer, builder.build().iter(), stroke.fill, 0.1);
            return; // We drew the full silhouette, done.
        }

        // 2. Render Fill
        if let Some(color) = self.fill {
            // Shrink fill slightly to hide anti-aliasing seams
            let fill_r = if stroke_width > 0.0 {
                r - stroke_width.min(0.5)
            } else {
                r
            };

            if fill_r > 0.1 {
                let mut builder = Path::builder();
                builder.add_circle(point(cx, cy), fill_r, lyon::path::Winding::Positive);
                tess.fill_path(buffer, builder.build().iter(), color, 0.1);
            }
        }

        // 3. Render Stroke
        if let Some(stroke) = &self.stroke {
            let width = stroke_width;

            // Inset Logic: Radius = R - Width/2
            let stroke_radius = r - (width / 2.0);

            // If we are here, we passed the "Safety Measure" (width < r),
            // so stroke_radius should be positive (at least r/2).
            if stroke_radius > 0.1 {
                let mut builder = Path::builder();
                builder.add_circle(point(cx, cy), stroke_radius, lyon::path::Winding::Positive);

                tess.stroke_path(buffer, builder.build().iter(), stroke, width, 0.1);
            }
        }
    }
}
