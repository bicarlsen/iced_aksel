use aksel::{Float, PlotPoint, Transform};
use iced::Color;
use lyon::math::{Box2D, Point, Vector, point};

use crate::{
    Length, Shape, Stroke, plot,
    render::{MeshBuffer, Tessellators},
};

#[derive(Debug, Clone)]
pub struct Polygon<D> {
    points: Vec<PlotPoint<D>>,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Polygon<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.lyon_tessellation(transform, buffer, tess);
        })
    }
}

impl<D: Float> Polygon<D> {
    pub const fn new(points: Vec<PlotPoint<D>>) -> Self {
        Self {
            points,
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
        if self.points.len() < 3 {
            return;
        }

        // 1. Convert to Screen Space
        let mut screen_points: Vec<Point> = self
            .points
            .iter()
            .map(|p| point(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)))
            .collect();

        // Ensure Counter-Clockwise Winding for consistent math
        if self.signed_area(&screen_points) > 0.0 {
            screen_points.reverse();
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

        // 2. Render Fill
        if let Some(color) = self.fill {
            let fill_points = if stroke_width > 0.0 {
                // Shrink fill slightly (0.5px) to hide AA gaps
                self.compute_inset_polygon(&screen_points, stroke_width.min(0.5))
                    .unwrap_or_else(|| screen_points.clone())
            } else {
                screen_points.clone()
            };

            tess.fill_polygon(buffer, fill_points.iter().cloned(), color);
        }

        // 3. Render Stroke
        if let Some(stroke) = &self.stroke {
            let half_width = stroke_width / 2.0;

            match self.compute_inset_polygon(&screen_points, half_width) {
                Some(inset_points) => {
                    // Success! Draw the inner path.
                    tess.stroke_polyline(buffer, inset_points, stroke, stroke_width, true);
                }
                None => {
                    // Failure: Stroke is too thick for the shape (Consumed).
                    // Render the full silhouette as a solid shape using the stroke color.
                    // This ensures "size 1" remains "size 1" visually.
                    tess.fill_polygon(buffer, screen_points.iter().cloned(), stroke.fill);
                }
            }
        }
    }

    fn signed_area(&self, points: &[Point]) -> f32 {
        let last = points.last().unwrap();
        (points[0].x - last.x).mul_add(
            points[0].y + last.y,
            points
                .windows(2)
                .map(|w| (w[1].x - w[0].x) * (w[1].y + w[0].y))
                .sum::<f32>(),
        )
    }

    fn compute_inset_polygon(&self, points: &[Point], distance: f32) -> Option<Vec<Point>> {
        let len = points.len();
        let mut new_points = Vec::with_capacity(len);

        // Safety 1: Calculate Original Size
        let original_bounds = Box2D::from_points(points.iter().cloned());
        let orig_diag_sq = (original_bounds.max - original_bounds.min).square_length();

        for i in 0..len {
            let p_prev = points[(i + len - 1) % len];
            let p_curr = points[i];
            let p_next = points[(i + 1) % len];

            let v1 = (p_curr - p_prev).normalize();
            let v2 = (p_next - p_curr).normalize();

            // Check degenerate edges
            if v1.x.is_nan() || v2.x.is_nan() {
                return None;
            }

            let n1 = Vector::new(-v1.y, v1.x);

            let p1_shift = p_prev + n1 * distance;
            let p2_shift = p_curr + n1 * distance;

            let n2 = Vector::new(-v2.y, v2.x);
            let p3_shift = p_curr + n2 * distance;
            let p4_shift = p_next + n2 * distance;

            if let Some(intersect) = self.line_intersection(p1_shift, p2_shift, p3_shift, p4_shift)
            {
                new_points.push(intersect);
            } else {
                new_points.push(p2_shift);
            }
        }

        // Safety 2: Winding Flip
        let original_area = self.signed_area(points);
        let new_area = self.signed_area(&new_points);

        if original_area.signum() != new_area.signum() || new_area.abs() < 1.0 {
            return None;
        }

        // Safety 3: Bounding Box Explosion (The Fix for "20 Big")
        // If we are shrinking a polygon, it should generally get smaller.
        // If the resulting polygon is LARGER than the original, it means the lines crossed over
        // and created an "inverted" shape on the outside.
        let new_bounds = Box2D::from_points(new_points.iter().cloned());
        let new_diag_sq = (new_bounds.max - new_bounds.min).square_length();

        // Use a small epsilon to avoid floating point noise issues when size is unchanged
        if new_diag_sq > orig_diag_sq + 1.0 {
            return None;
        }

        Some(new_points)
    }

    fn line_intersection(&self, p1: Point, p2: Point, p3: Point, p4: Point) -> Option<Point> {
        let x1 = p1.x;
        let y1 = p1.y;
        let x2 = p2.x;
        let y2 = p2.y;
        let x3 = p3.x;
        let y3 = p3.y;
        let x4 = p4.x;
        let y4 = p4.y;

        let denom = (y4 - y3).mul_add(x2 - x1, -((x4 - x3) * (y2 - y1)));

        if denom.abs() < 1e-5 {
            return None;
        }

        let ua = (x4 - x3).mul_add(y1 - y3, -((y4 - y3) * (x1 - x3))) / denom;

        Some(point(ua.mul_add(x2 - x1, x1), ua.mul_add(y2 - y1, y1)))
    }
}
