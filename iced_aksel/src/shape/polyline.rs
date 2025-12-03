use crate::{
    Length, Shape, Stroke, StrokeStyle,
    plot::{self},
    render::{MeshBuffer, Tessellators},
};
use aksel::{Float, PlotPoint, Transform};
use iced::{
    Color, Rectangle,
    advanced::graphics::{color::pack, mesh::SolidVertex2D},
};
use lyon::math::{Point, Vector};

/// A connected series of line segments.
///
/// Supports infinite extension on the first/last segments, optional arrowheads,
/// and proper miter joins via Lyon tessellation.
#[derive(Debug, Clone)]
pub struct Polyline<D> {
    pub points: Vec<PlotPoint<D>>,
    pub stroke: Stroke<D>,
    pub extend_start: bool,
    pub extend_end: bool,
    pub arrow_start: bool,
    pub arrow_end: bool,
    pub arrow_size: f32,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Polyline<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Polyline<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    /// Creates a new Polyline from a list of points.
    pub fn new(points: Vec<PlotPoint<D>>, stroke: Stroke<D>) -> Self {
        Self {
            points,
            stroke,
            extend_start: false,
            extend_end: false,
            arrow_start: false,
            arrow_end: false,
            arrow_size: 3.0,
        }
    }

    // =========================================================================
    //  Builder Methods
    // =========================================================================

    pub fn extend_start(mut self, enable: bool) -> Self {
        self.extend_start = enable;
        self
    }

    pub fn extend_end(mut self, enable: bool) -> Self {
        self.extend_end = enable;
        self
    }

    pub fn arrow_start(mut self, enable: bool) -> Self {
        self.arrow_start = enable;
        self
    }

    pub fn arrow_end(mut self, enable: bool) -> Self {
        self.arrow_end = enable;
        self
    }

    pub fn arrow_size(mut self, size: f32) -> Self {
        self.arrow_size = size;
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
        if self.points.len() < 2 {
            return;
        }

        // 1. Resolve to Screen Coordinates
        // Optimization: Pre-allocate vector
        let mut screen_points: Vec<Point> = self
            .points
            .iter()
            .map(|p| Point::new(transform.x_to_screen(&p.x), transform.y_to_screen(&p.y)))
            .collect();

        // Cull if degenerate (all points identical?)
        // Simple check on bounding box could go here, but Lyon handles it.

        // 2. Resolve Stroke Thickness
        let width = match self.stroke.thickness {
            Length::Screen(w) => w,
            Length::Plot(w) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&w);
                (p1 - p0).abs()
            }
        };

        if width < 0.1 {
            return;
        }

        let bounds = transform.screen_bounds();
        // Margin for clipping to ensure arrows/caps don't disappear
        let clip_margin = width * self.arrow_size.max(2.0);
        let clip_rect = (
            bounds.x - clip_margin,
            bounds.y - clip_margin,
            bounds.x + bounds.width + clip_margin,
            bounds.y + bounds.height + clip_margin,
        );

        // 3. Handle Arrowhead Retraction & Extension
        // We modify the FIRST and LAST points of the vector.
        let first_idx = 0;
        let last_idx = screen_points.len() - 1;

        // --- Start Segment Logic ---
        let mut p0 = screen_points[first_idx];
        let p1 = screen_points[first_idx + 1];
        let start_dir = (p1 - p0).normalize();

        let arrow_len = width * self.arrow_size;

        if self.extend_start {
            // Extend backwards to screen edge
            if let Some((t0, _)) = clip_line(p0, p1, clip_rect) {
                // t0 is the entry point (likely negative if extending backwards)
                // P_new = P0 + t0 * (P1 - P0)
                let delta = p1 - p0;
                // Only extend if t0 < 0 (i.e., the edge is "behind" P0)
                if t0 < 0.0 {
                    screen_points[first_idx] = p0 + delta * t0;
                }
            }
        } else if self.arrow_start {
            // Retract forwards to make room for arrow base
            screen_points[first_idx] = p0 + start_dir * arrow_len;
        }

        // --- End Segment Logic ---
        let mut pn = screen_points[last_idx];
        let pn_minus_1 = screen_points[last_idx - 1];
        let end_dir = (pn - pn_minus_1).normalize();

        if self.extend_end {
            // Extend forwards to screen edge
            if let Some((_, t1)) = clip_line(pn_minus_1, pn, clip_rect) {
                // t1 is the exit point
                let delta = pn - pn_minus_1;
                // Only extend if t1 > 1.0 (i.e., the edge is "ahead" of Pn)
                if t1 > 1.0 {
                    screen_points[last_idx] = pn_minus_1 + delta * t1;
                }
            }
        } else if self.arrow_end {
            // Retract backwards
            screen_points[last_idx] = pn - end_dir * arrow_len;
        }

        // 4. Render Body (Lyon)
        // We use Lyon for the body to ensure proper joins between segments.
        tess.stroke_polyline(
            buffer,
            screen_points, // Pass ownership of the vector
            &self.stroke,
            width,
            false, // Open path
        );

        // 5. Render Arrowheads (Manual)
        // Draw at ORIGINAL locations (p0, pn) not the retracted/extended ones.
        // Unless extended (infinity -> no arrow).

        if self.arrow_start && !self.extend_start {
            // Pointing backwards from p0. Direction is -start_dir.
            self.add_arrowhead(buffer, p0, -start_dir, width, self.stroke.fill);
        }

        if self.arrow_end && !self.extend_end {
            // Pointing forwards at pn. Direction is end_dir.
            self.add_arrowhead(buffer, pn, end_dir, width, self.stroke.fill);
        }
    }

    // --- Helpers ---

    fn add_arrowhead(
        &self,
        buffer: &mut MeshBuffer,
        tip: Point,
        direction: Vector,
        width: f32,
        color: Color,
    ) {
        let c = pack(color);

        let arrow_len = width * self.arrow_size;
        let arrow_width = width * self.arrow_size * 0.8;

        let base_center = tip - direction * arrow_len;
        let normal = Vector::new(-direction.y, direction.x) * (arrow_width / 2.0);
        let wing1 = base_center + normal;
        let wing2 = base_center - normal;

        buffer.add(
            &[0, 1, 2],
            &[
                SolidVertex2D {
                    position: tip.to_array(),
                    color: c,
                },
                SolidVertex2D {
                    position: wing1.to_array(),
                    color: c,
                },
                SolidVertex2D {
                    position: wing2.to_array(),
                    color: c,
                },
            ],
        );
    }
}

// --- Math Utilities (Reused) ---

fn clip_line(p1: Point, p2: Point, clip_rect: (f32, f32, f32, f32)) -> Option<(f32, f32)> {
    let (xmin, ymin, xmax, ymax) = clip_rect;
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;

    let mut t0 = -100_000.0;
    let mut t1 = 100_000.0;

    let p = [-dx, dx, -dy, dy];
    let q = [p1.x - xmin, xmax - p1.x, p1.y - ymin, ymax - p1.y];

    for i in 0..4 {
        if p[i].abs() < 1e-6 {
            if q[i] < 0.0 {
                return None;
            }
        } else {
            let t = q[i] / p[i];
            if p[i] < 0.0 {
                if t > t1 {
                    return None;
                }
                if t > t0 {
                    t0 = t;
                }
            } else {
                if t < t0 {
                    return None;
                }
                if t < t1 {
                    t1 = t;
                }
            }
        }
    }

    if t0 <= t1 { Some((t0, t1)) } else { None }
}
