use aksel::{Float, PlotPoint, Transform};
use lyon::math::point;

use crate::{
    Length, Shape, Stroke, plot,
    render::{MeshBuffer, Tessellators},
};

#[derive(Debug, Clone, Copy)]
pub struct Line<D> {
    pub start: PlotPoint<D>,
    pub end: PlotPoint<D>,
    pub stroke: Stroke<D>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Line<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.lyon_tessellation(transform, buffer, tess);
        })
    }
}

impl<D: Float> Line<D> {
    pub const fn new(start: PlotPoint<D>, end: PlotPoint<D>, stroke: Stroke<D>) -> Self {
        Self { start, end, stroke }
    }

    fn lyon_tessellation(
        self,
        transform: &Transform<D, D, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        let line_width = match self.stroke.thickness {
            Length::Screen(width) => width,
            Length::Plot(width) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&width);
                (p1 - p0).abs()
            }
        };

        // Skip degenerate lines that collapse to a point.
        if self.start.x == self.end.x && self.start.y == self.end.y {
            return;
        }

        let start = point(
            transform.x_to_screen(&self.start.x),
            transform.y_to_screen(&self.start.y),
        );
        let end = point(
            transform.x_to_screen(&self.end.x),
            transform.y_to_screen(&self.end.y),
        );

        tess.stroke_polyline(buffer, [start, end], &self.stroke, line_width, false);
    }
}
