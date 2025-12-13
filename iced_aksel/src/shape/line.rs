use crate::{
    Float, Measure, PlotPoint, Shape, Stroke, Transform,
    plot::{self},
    render::{MeshBuffer, Tessellators},
    stroke::StrokeStyle,
};

use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};
use lyon::math::{Point, Vector};

/// A line segment defined by two points.
///
/// Supports infinite extension in either direction (Ray or Line), optional arrowheads,
/// and utilizes a Hybrid Engine for high-performance rendering.
///
/// # Example
///
/// ```rust
/// use iced_aksel::{PlotPoint, Measure, shape::Line, Stroke};
/// use iced::Color;
///
/// // Simple line from (0, 0) to (10, 10)
/// let line = Line::new(
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(10.0, 10.0),
///     Stroke::new(Color::from_rgb(0.0, 0.0, 1.0), Measure::Screen(2.0))
/// );
///
/// // Line with arrow at the end
/// let arrow_line = Line::new(
///     PlotPoint::new(5.0, 5.0),
///     PlotPoint::new(15.0, 15.0),
///     Stroke::new(Color::from_rgb(1.0, 0.0, 0.0), Measure::Screen(3.0))
/// ).arrow_end(true);
///
/// // Infinite line through two points
/// let infinite = Line::new(
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(1.0, 1.0),
///     Stroke::new(Color::from_rgb(0.0, 1.0, 0.0), Measure::Screen(1.0))
/// ).infinite();
/// ```
#[derive(Debug, Clone)]
pub struct Line<D> {
    pub p1: PlotPoint<D>,
    pub p2: PlotPoint<D>,
    pub stroke: Stroke<D>,
    pub extend_start: bool,
    pub extend_end: bool,
    pub arrow_start: bool,
    pub arrow_end: bool,
    pub arrow_size: f32, // Multiplier of stroke width
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Line<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Line<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    /// Creates a new Line segment between two points.
    pub const fn new(p1: PlotPoint<D>, p2: PlotPoint<D>, stroke: Stroke<D>) -> Self {
        Self {
            p1,
            p2,
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

    /// Extends the line infinitely in the direction of the start point (P1).
    /// Note: This disables `arrow_start`.
    pub const fn extend_start(mut self, enable: bool) -> Self {
        self.extend_start = enable;
        self
    }

    /// Extends the line infinitely in the direction of the end point (P2).
    /// Note: This disables `arrow_end`.
    pub const fn extend_end(mut self, enable: bool) -> Self {
        self.extend_end = enable;
        self
    }

    /// Extends the line infinitely in both directions.
    pub const fn infinite(mut self) -> Self {
        self.extend_start = true;
        self.extend_end = true;
        self
    }

    /// Draws an arrowhead at the start point (P1).
    pub const fn arrow_start(mut self, enable: bool) -> Self {
        self.arrow_start = enable;
        self
    }

    /// Draws an arrowhead at the end point (P2).
    pub const fn arrow_end(mut self, enable: bool) -> Self {
        self.arrow_end = enable;
        self
    }

    /// Draws arrowheads at both ends.
    pub const fn arrows(mut self, enable: bool) -> Self {
        self.arrow_start = enable;
        self.arrow_end = enable;
        self
    }

    /// Sets the size of the arrowhead relative to the stroke thickness.
    /// Default is 3.0.
    pub const fn arrow_size(mut self, multiplier: f32) -> Self {
        self.arrow_size = multiplier;
        self
    }

    // =========================================================================
    //  Tessellation Logic
    // =========================================================================

    fn tessellate(
        self,
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        // 1. Resolve to Screen Coordinates
        let raw_start = Point::new(
            transform.x_to_screen(&self.p1.x),
            transform.y_to_screen(&self.p1.y),
        );
        let raw_end = Point::new(
            transform.x_to_screen(&self.p2.x),
            transform.y_to_screen(&self.p2.y),
        );

        // Cull zero-length lines
        let dir_vec = raw_end - raw_start;
        if dir_vec.square_length() < 0.001 {
            return;
        }
        let dir = dir_vec.normalize();

        // 2. Resolve Stroke Thickness
        let width = match self.stroke.thickness {
            Measure::Screen(w) => w,
            Measure::Plot(w) => {
                let p0 = transform.x_to_screen(&D::zero());
                let p1 = transform.x_to_screen(&w);
                (p1 - p0).abs()
            }
        };

        if width < 0.1 {
            return;
        }

        // 3. Handle Arrowhead Retraction
        // We must shorten the line segment so it stops at the BASE of the arrow,
        // not the TIP. This prevents the "blocky" line from poking through the arrow.

        let arrow_len = width * self.arrow_size;
        let mut line_start = raw_start;
        let mut line_end = raw_end;

        // If we have an arrow AND we are not infinite in that direction:
        // Shorten the line.
        if self.arrow_start && !self.extend_start {
            line_start = raw_start + dir * arrow_len;
        }
        if self.arrow_end && !self.extend_end {
            line_end = raw_end - dir * arrow_len;
        }

        // Safety: If the line is so short that the arrows overlap/cross,
        // we shouldn't draw the line segment at all (it's inside the arrows).
        // Check if the direction from new start to new end is opposite to original dir.
        let segment_valid = (line_end - line_start).dot(dir) > 0.0;

        // 4. Clipping & Extension (Liang-Barsky)
        // We clip the *shortened* line segment (line_start -> line_end).
        // If extend_start is true, we ignore the shortening (handled by logic above)
        // and let the clipper extend to the screen edge.

        let bounds = transform.screen_bounds();
        let clip_margin = width * self.arrow_size.max(1.0);
        let clip_rect = (
            bounds.x - clip_margin,
            bounds.y - clip_margin,
            bounds.x + bounds.width + clip_margin,
            bounds.y + bounds.height + clip_margin,
        );

        let mut draw_start = line_start;
        let mut draw_end = line_end;
        let mut is_visible = true;

        // Note: We use raw_start/raw_end for the *direction* and *origin* of infinite lines
        // but we use line_start/line_end for the *finite* bounds.
        // Simplified: Just clip the segment we calculated.
        // If it's infinite, we must manually extend BEFORE clipping?
        // No, Liang-Barsky takes a line segment P1->P2.
        // To handle infinity correctly with the 'shortened' logic:

        // Revert to raw points for the Infinite side calculation
        let p1 = if self.extend_start {
            raw_start
        } else {
            line_start
        };
        let p2 = if self.extend_end { raw_end } else { line_end };

        if let Some((t0, t1)) = clip_line(p1, p2, clip_rect) {
            let delta = p2 - p1;

            // t0/t1 are relative to the P1->P2 vector.
            if self.extend_start {
                draw_start = p1 + delta * t0;
            } else {
                // For finite start, only clip if it was offscreen (t0 > 0)
                if t0 > 0.0 {
                    draw_start = p1 + delta * t0;
                } else {
                    draw_start = p1;
                }
            }

            if self.extend_end {
                draw_end = p1 + delta * t1;
            } else {
                // For finite end, only clip if it was offscreen (t1 < 1)
                if t1 < 1.0 {
                    draw_end = p1 + delta * t1;
                } else {
                    draw_end = p2;
                }
            }
        } else {
            is_visible = false;
        }

        // 5. Render Line (Hybrid Engine)
        if is_visible && segment_valid {
            match self.stroke.style {
                StrokeStyle::Solid => {
                    self.add_solid_segment(buffer, draw_start, draw_end, width, self.stroke.fill);
                }
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    let points = vec![draw_start, draw_end];
                    tess.stroke_polyline(
                        buffer,
                        points,
                        &self.stroke,
                        width,
                        false, // Open path
                    );
                }
            }
        }

        // 6. Render Arrowheads
        // Arrows are always drawn at the ORIGINAL points (raw_start/raw_end).
        // (Unless clipped off screen)

        // Direction vector for orientation
        // We use the original direction (raw_end - raw_start) to ensure arrows point
        // correctly even if the line segment was fully consumed/hidden.

        if self.arrow_start && !self.extend_start {
            // Pointing backwards from p1
            self.add_arrowhead(buffer, raw_start, -dir, width, self.stroke.fill);
        }

        if self.arrow_end && !self.extend_end {
            // Pointing forwards at p2
            self.add_arrowhead(buffer, raw_end, dir, width, self.stroke.fill);
        }
    }

    // --- Helpers ---

    /// Adds a thick line segment (quad) to the mesh.
    fn add_solid_segment(
        &self,
        buffer: &mut MeshBuffer,
        p1: Point,
        p2: Point,
        width: f32,
        color: Color,
    ) {
        let c = pack(color);
        let half_width = width / 2.0;

        let dir = (p2 - p1).normalize();
        let normal = Vector::new(-dir.y, dir.x) * half_width;

        let c1 = p1 + normal;
        let c2 = p1 - normal;
        let c3 = p2 + normal;
        let c4 = p2 - normal;

        buffer.add(
            &[0, 1, 2, 1, 2, 3],
            &[
                SolidVertex2D {
                    position: c1.to_array(),
                    color: c,
                }, // TL
                SolidVertex2D {
                    position: c3.to_array(),
                    color: c,
                }, // TR
                SolidVertex2D {
                    position: c2.to_array(),
                    color: c,
                }, // BL
                SolidVertex2D {
                    position: c4.to_array(),
                    color: c,
                }, // BR
            ],
        );
    }

    /// Adds a triangular arrowhead.
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

// --- Math Utilities ---

/// Liang-Barsky line clipping algorithm.
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
