use crate::{
    Measure, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellators},
    stroke::StrokeStyle,
};
use aksel::{Float, PlotPoint, Transform};
use iced_core::Color;
use iced_graphics::{color::pack, mesh::SolidVertex2D};
use lyon::math::{Point, Vector};

/// Internal storage for the triangle's definition.
#[derive(Debug, Clone)]
enum Geometry<D> {
    /// Explicit points (Static in Data Space)
    Vertices([PlotPoint<D>; 3]),
    /// Defined by center and radius (Dynamic resolution)
    Equilateral {
        center: PlotPoint<D>,
        radius: Measure<D>,
    },
}

/// A triangle defined by three points or a center/radius.
///
/// This shape supports both generic triangles (via [`Triangle::new`]) and
/// equilateral triangles (via [`Triangle::equilateral`]).
///
/// It utilizes a Hybrid Engine:
/// - **Fill & Solid Stroke:** Uses manual vertex generation with miter-adjusted insets. (very fast)
/// - **Dashed/Dotted:** Falls back to Lyon (a lot slower at rendering tens of thousands).
///
/// # Example
///
/// ```rust
/// use iced_aksel::{PlotPoint, Measure, shape::Triangle, Stroke};
/// use iced::Color;
///
/// // Custom triangle from three points
/// let tri = Triangle::new(
///     PlotPoint::new(0.0, 0.0),
///     PlotPoint::new(10.0, 0.0),
///     PlotPoint::new(5.0, 10.0)
/// ).fill(Color::from_rgb(1.0, 0.0, 0.0));
///
/// // Equilateral triangle (constant size marker)
/// let marker = Triangle::equilateral(
///     PlotPoint::new(25.0, 25.0),
///     Measure::Screen(8.0)
/// ).fill(Color::from_rgb(0.0, 1.0, 0.0));
/// ```
#[derive(Debug, Clone)]
pub struct Triangle<D> {
    geometry: Geometry<D>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Triangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Triangle<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    /// Creates a generic triangle from three points in Plot Space. This will always scale with
    /// chart zoom
    pub const fn new(p1: PlotPoint<D>, p2: PlotPoint<D>, p3: PlotPoint<D>) -> Self {
        Self {
            geometry: Geometry::Vertices([p1, p2, p3]),
            fill: None,
            stroke: None,
        }
    }

    /// Creates an equilateral triangle defined by a center and a radius.
    ///
    /// The triangle points upwards (one vertex at 12 o'clock).
    ///
    /// - If `radius` is [`Measure::Screen`], the triangle will maintain constant pixel size.
    /// - If `radius` is [`Measure::Plot`], the triangle will scale with the chart zoom.
    pub const fn equilateral(center: PlotPoint<D>, radius: Measure<D>) -> Self {
        Self {
            geometry: Geometry::Equilateral { center, radius },
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
        transform: &Transform<D, f32, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        // 1. Resolve to Screen Coordinates
        let (v1, v2, v3) = match self.geometry {
            Geometry::Vertices(pts) => {
                // Standard mapping for raw points
                (
                    Point::new(
                        transform.x_to_screen(&pts[0].x),
                        transform.y_to_screen(&pts[0].y),
                    ),
                    Point::new(
                        transform.x_to_screen(&pts[1].x),
                        transform.y_to_screen(&pts[1].y),
                    ),
                    Point::new(
                        transform.x_to_screen(&pts[2].x),
                        transform.y_to_screen(&pts[2].y),
                    ),
                )
            }
            Geometry::Equilateral { center, radius } => {
                // Dynamic mapping for equilateral
                let cx = transform.x_to_screen(&center.x);
                let cy = transform.y_to_screen(&center.y);

                // Resolve Radius to Pixels (Screen vs Plot)
                let r_px = match radius {
                    Measure::Screen(px) => px,
                    Measure::Plot(units) => {
                        let p0 = transform.x_to_screen(&D::zero());
                        let p1 = transform.x_to_screen(&units);
                        (p1 - p0).abs()
                    }
                };

                // Calculate vertices in Screen Space
                // Angles: 90 (Up), 210 (Bottom Left), 330 (Bottom Right)
                // Note: Screen Y is often inverted (0 at top), so "Up" might be -Y.
                // Assuming standard Math coords: sin(90) = 1.
                // If Screen Y increases downwards: sin(90) = 1 -> Down.
                // To point "Up" visually on screen, we typically want -Y.
                // Let's stick to standard unit circle angles and let the user rotate if needed,
                // or just use -PI/2 for "Up" in screen space.

                // Using standard geometric angles (CCW from X-axis):
                // 90 deg = PI/2
                // 210 deg = 7PI/6
                // 330 deg = 11PI/6

                // We use f32 directly here since we are in screen space
                let a1 = std::f32::consts::FRAC_PI_2;
                let a2 = std::f32::consts::PI * 7.0 / 6.0;
                let a3 = std::f32::consts::PI * 11.0 / 6.0;

                // Note: In screen space, -Y is Up.
                // sin(90) = 1. So (0, r) is Down.
                // To point Up, we negate the Y component or shift angles.
                // Let's just generate them:

                let p1 = Point::new(a1.cos().mul_add(r_px, cx), a1.sin().mul_add(-r_px, cy)); // Up
                let p2 = Point::new(a2.cos().mul_add(r_px, cx), a2.sin().mul_add(-r_px, cy)); // Left Down
                let p3 = Point::new(a3.cos().mul_add(r_px, cx), a3.sin().mul_add(-r_px, cy)); // Right Down

                (p1, p2, p3)
            }
        };

        // 2. Normalize Winding Order (Calculate Area)
        // We want Counter-Clockwise (CCW).
        // Cross product (z-component): (p2-p1) x (p3-p1)
        let cross_raw = (v2.x - v1.x).mul_add(v3.y - v1.y, -((v2.y - v1.y) * (v3.x - v1.x)));

        // If cross < 0, it's CW. Swap vertices to make CCW.
        // We also keep the absolute area (2x Area) for the Inradius check later.
        let (p1, p2, p3, double_area) = if cross_raw < 0.0 {
            (v1, v3, v2, -cross_raw)
        } else {
            (v1, v2, v3, cross_raw)
        };

        // 3. Resolve Stroke
        let maybe_stroke_data = self.stroke.as_ref().and_then(|stroke| {
            let width = match stroke.thickness {
                Measure::Screen(w) => w,
                Measure::Plot(w) => {
                    let p0 = transform.x_to_screen(&D::zero());
                    let p1 = transform.x_to_screen(&w);
                    (p1 - p0).abs()
                }
            };
            if width < 0.1 {
                None
            } else {
                Some((width, stroke))
            }
        });

        // 4. Pre-calculate Inset Vertices (Geometric Consumption Check)
        let (inner_p1, inner_p2, inner_p3, is_consumed) =
            if let Some((width, _)) = maybe_stroke_data {
                // GEOMETRIC CHECK: Inradius
                // The largest circle that fits inside a triangle has radius r = 2 * Area / Perimeter.
                let d1 = (p2 - p1).length();
                let d2 = (p3 - p2).length();
                let d3 = (p1 - p3).length();
                let perimeter = d1 + d2 + d3;

                // Degenerate triangle check
                if perimeter < 1e-4 {
                    (
                        Point::new(0.0, 0.0),
                        Point::new(0.0, 0.0),
                        Point::new(0.0, 0.0),
                        true,
                    )
                } else {
                    let inradius = double_area / perimeter;

                    if width >= inradius {
                        // Consumed by stroke size
                        (
                            Point::new(0.0, 0.0),
                            Point::new(0.0, 0.0),
                            Point::new(0.0, 0.0),
                            true,
                        )
                    } else {
                        // Not consumed, safe to calculate geometry
                        let i1 = self.compute_inset_vertex(p3, p1, p2, width);
                        let i2 = self.compute_inset_vertex(p1, p2, p3, width);
                        let i3 = self.compute_inset_vertex(p2, p3, p1, width);

                        // Final Sanity Check
                        let inner_cross =
                            (i2.x - i1.x).mul_add(i3.y - i1.y, -((i2.y - i1.y) * (i3.x - i1.x)));
                        let inverted = inner_cross <= 0.0;

                        (i1, i2, i3, inverted)
                    }
                }
            } else {
                (p1, p2, p3, false)
            };

        // FAST PATH: Consumed
        if is_consumed {
            if let Some((_, stroke)) = maybe_stroke_data {
                self.add_solid_triangle(buffer, p1, p2, p3, stroke.fill);
            }
            return;
        }

        // 5. Render Fill
        if let Some(color) = self.fill {
            if maybe_stroke_data.is_some() {
                // Rule 3: Bleed Fix
                let d = 0.5;
                let f1 = self.compute_inset_vertex(p3, p1, p2, d);
                let f2 = self.compute_inset_vertex(p1, p2, p3, d);
                let f3 = self.compute_inset_vertex(p2, p3, p1, d);
                self.add_solid_triangle(buffer, f1, f2, f3, color);
            } else {
                self.add_solid_triangle(buffer, p1, p2, p3, color);
            }
        }

        // 6. Render Stroke
        if let Some((width, stroke)) = maybe_stroke_data {
            match stroke.style {
                StrokeStyle::Solid => {
                    // MANUAL PATH
                    self.add_manual_stroke(
                        buffer,
                        [p1, p2, p3],
                        [inner_p1, inner_p2, inner_p3],
                        stroke.fill,
                    );
                }
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    // LYON PATH
                    let d = width / 2.0;
                    let c1 = self.compute_inset_vertex(p3, p1, p2, d);
                    let c2 = self.compute_inset_vertex(p1, p2, p3, d);
                    let c3 = self.compute_inset_vertex(p2, p3, p1, d);

                    let points = vec![c1, c2, c3, c1];
                    tess.stroke_polyline(buffer, points, stroke, width, true);
                }
            }
        }
    }

    // --- Math Helpers ---

    /// Computes the new location of `current` when the edges connecting it
    /// to `prev` and `next` are shifted inward by `distance`.
    fn compute_inset_vertex(
        &self,
        prev: Point,
        current: Point,
        next: Point,
        distance: f32,
    ) -> Point {
        let v1 = (current - prev).normalize();
        let v2 = (next - current).normalize();

        let tangent = (v1 + v2).normalize();
        let miter = Vector::new(-tangent.y, tangent.x);
        let n1 = Vector::new(-v1.y, v1.x);

        let miter_len = distance / miter.dot(n1);
        let limited_len = miter_len.min(distance * 5.0);

        current + miter * limited_len
    }

    // --- Manual Tessellation Writers ---

    fn add_solid_triangle(
        &self,
        buffer: &mut MeshBuffer,
        p1: Point,
        p2: Point,
        p3: Point,
        color: Color,
    ) {
        let c = pack(color);
        buffer.add(
            &[0, 1, 2],
            &[
                SolidVertex2D {
                    position: p1.to_array(),
                    color: c,
                },
                SolidVertex2D {
                    position: p2.to_array(),
                    color: c,
                },
                SolidVertex2D {
                    position: p3.to_array(),
                    color: c,
                },
            ],
        );
    }

    fn add_manual_stroke(
        &self,
        buffer: &mut MeshBuffer,
        outer: [Point; 3],
        inner: [Point; 3],
        color: Color,
    ) {
        let c = pack(color);

        let vertices = [
            // Outer 0, 1, 2
            SolidVertex2D {
                position: outer[0].to_array(),
                color: c,
            },
            SolidVertex2D {
                position: outer[1].to_array(),
                color: c,
            },
            SolidVertex2D {
                position: outer[2].to_array(),
                color: c,
            },
            // Inner 3, 4, 5
            SolidVertex2D {
                position: inner[0].to_array(),
                color: c,
            },
            SolidVertex2D {
                position: inner[1].to_array(),
                color: c,
            },
            SolidVertex2D {
                position: inner[2].to_array(),
                color: c,
            },
        ];

        let indices = [
            0, 1, 4, 0, 4, 3, // Side 1
            1, 2, 5, 1, 5, 4, // Side 2
            2, 0, 3, 2, 3, 5, // Side 3
        ];

        buffer.add(&indices, &vertices);
    }
}
