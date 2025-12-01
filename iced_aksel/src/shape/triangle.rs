use aksel::{Float, PlotPoint, Transform};
use iced::Color;
use lyon::math::point;

use crate::{
    Length, Shape, Stroke, plot,
    render::{MeshBuffer, Tessellators},
};

#[derive(Debug, Clone)]
pub struct Triangle<D> {
    center: PlotPoint<D>,
    size: Length<D>,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Triangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.lyon_tessellation(transform, buffer, tess);
        })
    }
}

impl<D: Float> Triangle<D> {
    pub const fn new(center: PlotPoint<D>, size: Length<D>) -> Self {
        Self {
            center,
            size,
            fill: None,
            stroke: None,
        }
    }

    pub fn from_points(points: [PlotPoint<D>; 3]) -> Self {
        let three = D::from(3).unwrap();
        let center = PlotPoint::new(
            (points[0].x + points[1].x + points[2].x) / three,
            (points[0].y + points[1].y + points[2].y) / three,
        );
        let size = Length::Plot((points[0].x - points[1].x).abs());
        Self {
            center,
            size,
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

        let r = match self.size {
            Length::Screen(px) => px,
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
                    let x0 = transform.x_to_screen(&D::zero());
                    let x1 = transform.x_to_screen(&w);
                    (x1 - x0).abs()
                }
            });

        // SAFETY MEASURE:
        // For an equilateral triangle, the distance from center to edge (apothem) is r * 0.5.
        // If the stroke width is larger than the apothem, the stroke completely covers the center.
        // Trying to stroke a tiny path with a massive pen causes miter artifacts ("spinning rectangles").
        // Therefore, if width > r * 0.5, we just render a solid triangle.
        // We use 0.6 as a conservative threshold to ensure the hole is fully closed before switching.
        if stroke_width >= r * 0.6
            && let Some(stroke) = &self.stroke
        {
            // Render the silhouette as a solid shape using the stroke color
            let points = self.compute_vertices(cx, cy, r);
            tess.fill_polygon(buffer, points.iter().cloned(), stroke.fill);
            return;
        }

        // 2. Render Fill
        if let Some(color) = self.fill {
            let fill_r = if stroke_width > 0.0 {
                r - stroke_width.min(0.5)
            } else {
                r
            };

            let points = self.compute_vertices(cx, cy, fill_r);
            tess.fill_polygon(buffer, points.iter().cloned(), color);
        }

        // 3. Render Stroke
        if let Some(stroke) = &self.stroke {
            // INSET LOGIC: R_new = R_old - width
            let stroke_r = r - stroke_width;

            // Since we already handled the "massive stroke" case above,
            // stroke_r is guaranteed to be reasonably positive here.
            if stroke_r > 0.1 {
                let points = self.compute_vertices(cx, cy, stroke_r);
                tess.stroke_polyline(buffer, points, stroke, stroke_width, true);
            }
        }
    }

    #[inline]
    fn compute_vertices(&self, cx: f32, cy: f32, r: f32) -> Vec<lyon::math::Point> {
        let angle_top = -std::f32::consts::FRAC_PI_2;
        let angle_br = std::f32::consts::PI / 6.0;
        let angle_bl = 5.0 * std::f32::consts::PI / 6.0;

        vec![
            point(
                r.mul_add(angle_top.cos(), cx),
                r.mul_add(angle_top.sin(), cy),
            ),
            point(r.mul_add(angle_br.cos(), cx), r.mul_add(angle_br.sin(), cy)),
            point(r.mul_add(angle_bl.cos(), cx), r.mul_add(angle_bl.sin(), cy)),
        ]
    }
}
