use aksel::{Float, PlotPoint, Transform};
use lyon::math::point;

use crate::{
    Length, Shape, Stroke, plot,
    render::{MeshBuffer, Tessellators},
};

#[derive(Debug, Clone)]
pub struct Polyline<D> {
    pub points: Vec<PlotPoint<D>>,
    pub stroke: Stroke<D>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Polyline<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.lyon_tessellation(transform, buffer, tess);
        })
    }
}

impl<D: Float> Polyline<D> {
    pub const fn new(points: Vec<PlotPoint<D>>, stroke: Stroke<D>) -> Self {
        Self { points, stroke }
    }

    fn lyon_tessellation(
        self,
        transform: &Transform<D, D, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        if self.points.len() < 2 {
            return;
        }

        let line_width = match self.stroke.thickness {
            Length::Screen(w) => w,
            Length::Plot(w) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&w);
                (p1 - p0).abs()
            }
        };

        // 1. Prepare Data
        let path_points = self
            .points
            .iter()
            .map(|p| point(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)));

        // 2. Execute
        // Clean, readable, logical ownership.
        tess.stroke_polyline(buffer, path_points, &self.stroke, line_width, false);
    }
}
