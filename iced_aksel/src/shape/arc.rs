use crate::{
    Length, Shape, Stroke,
    plot::{self},
    render::{MeshBuffer, Tessellators},
};
use aksel::{Float, PlotPoint, Transform};
use iced::{
    Color,
    advanced::graphics::{color::pack, mesh::SolidVertex2D},
};
use lyon::math::{Angle, Point, Vector};
use lyon::path::Path;

/// An arc (or pie slice/donut sector) defined by a center, radii, and angles.
///
/// - **Fill:** Uses a high-performance manual triangle strip generator.
/// - **Stroke:** Uses Lyon to handle complex line joins between straight and curved edges.
/// - **Coordinate System:** Math standard (0 is East, positive angles are Counter-Clockwise).
#[derive(Debug, Clone)]
pub struct Arc<D> {
    pub center: PlotPoint<D>,
    pub radius: Length<D>,
    pub inner_radius: Length<D>,
    pub start_angle: f32, // Radians
    pub end_angle: f32,   // Radians
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Arc<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Arc<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    /// Creates a new Arc/Pie Slice.
    pub const fn new(
        center: PlotPoint<D>,
        radius: Length<D>,
        start_angle: f32,
        end_angle: f32,
    ) -> Self {
        Self {
            center,
            radius,
            inner_radius: Length::Screen(0.0),
            start_angle,
            end_angle,
            fill: None,
            stroke: None,
        }
    }

    // =========================================================================
    //  Builder Methods
    // =========================================================================

    /// Sets the inner radius to create a Ring/Donut sector.
    pub fn inner_radius(mut self, radius: Length<D>) -> Self {
        self.inner_radius = radius;
        self
    }

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

        let outer_r = self.resolve_length(transform, self.radius);
        let inner_r = self.resolve_length(transform, self.inner_radius);

        if outer_r < 0.5 {
            return;
        }

        // 2. Resolve Stroke Thickness
        let maybe_stroke_data = if let Some(stroke) = &self.stroke {
            let width = self.resolve_length(transform, stroke.thickness);
            if width < 0.1 {
                None
            } else {
                Some((width, stroke))
            }
        } else {
            None
        };

        // 3. Rule 2: Geometric Stability (Consumption Check)
        let radial_thickness = outer_r - inner_r;
        let is_consumed = if let Some((width, _)) = maybe_stroke_data {
            width >= radial_thickness
        } else {
            false
        };

        if is_consumed {
            if let Some((_, stroke)) = maybe_stroke_data {
                self.add_solid_arc_strip(
                    buffer,
                    cx,
                    cy,
                    inner_r,
                    outer_r,
                    self.start_angle,
                    self.end_angle,
                    stroke.fill,
                );
            }
            return;
        }

        // 4. Render Fill (Manual Optimized Triangle Strip)
        if let Some(color) = self.fill {
            let mut draw_inner = inner_r;
            let mut draw_outer = outer_r;

            // Rule 3: Anti-Aliasing Polish (Bleed)
            if maybe_stroke_data.is_some() {
                draw_outer = (outer_r - 0.5).max(draw_inner);
                draw_inner = (inner_r + 0.5).min(draw_outer);
            }

            if (draw_outer - draw_inner) > 0.1 {
                self.add_solid_arc_strip(
                    buffer,
                    cx,
                    cy,
                    draw_inner,
                    draw_outer,
                    self.start_angle,
                    self.end_angle,
                    color,
                );
            }
        }

        // 5. Render Stroke (Lyon Path)
        if let Some((width, stroke)) = maybe_stroke_data {
            let center = Point::new(cx, cy);

            // Rule 1: Inner Stroke Alignment
            let s_inner = inner_r + width / 2.0;
            let s_outer = outer_r - width / 2.0;

            if s_outer <= s_inner {
                return;
            }

            // Fix 1: Check for Full Circle (Seam Fix)
            let sweep = (self.end_angle - self.start_angle).abs();
            let is_full_circle = sweep >= std::f32::consts::TAU - 0.001;

            let mut builder = Path::builder();

            if is_full_circle {
                // --- Full Circle Logic (Two separate rings) ---

                // 1. Outer Ring
                let outer_start = center + Vector::new(s_outer, 0.0);
                builder.begin(outer_start);
                let outer_arc = lyon::geom::Arc {
                    center,
                    radii: Vector::new(s_outer, s_outer),
                    start_angle: Angle::radians(0.0),
                    sweep_angle: Angle::radians(std::f32::consts::TAU),
                    x_rotation: Angle::radians(0.0),
                };
                outer_arc.for_each_cubic_bezier(&mut |segment| {
                    builder.cubic_bezier_to(segment.ctrl1, segment.ctrl2, segment.to);
                });
                builder.close();

                // 2. Inner Ring (Only if it's a donut, not a solid pie)
                if inner_r > 0.5 {
                    let inner_start = center + Vector::new(s_inner, 0.0);
                    builder.begin(inner_start);
                    let inner_arc = lyon::geom::Arc {
                        center,
                        radii: Vector::new(s_inner, s_inner),
                        start_angle: Angle::radians(0.0),
                        sweep_angle: Angle::radians(std::f32::consts::TAU),
                        x_rotation: Angle::radians(0.0),
                    };
                    inner_arc.for_each_cubic_bezier(&mut |segment| {
                        builder.cubic_bezier_to(segment.ctrl1, segment.ctrl2, segment.to);
                    });
                    builder.close();
                }
            } else {
                // --- Sector Logic (Connected shape) ---

                let start_cos = self.start_angle.cos();
                let start_sin = self.start_angle.sin();
                let end_cos = self.end_angle.cos();
                let end_sin = self.end_angle.sin();
                let sweep_a = Angle::radians(self.end_angle - self.start_angle);

                // Fix 2: Check for Pie Center (Artifact Fix)
                let is_pie_center = inner_r < 0.5;

                if is_pie_center {
                    // Path: Center -> OuterStart -> Arc -> Center -> Close

                    builder.begin(center);

                    let outer_start = center + Vector::new(start_cos, start_sin) * s_outer;
                    builder.line_to(outer_start);

                    let outer_arc = lyon::geom::Arc {
                        center,
                        radii: Vector::new(s_outer, s_outer),
                        start_angle: Angle::radians(self.start_angle),
                        sweep_angle: sweep_a,
                        x_rotation: Angle::radians(0.0),
                    };
                    outer_arc.for_each_cubic_bezier(&mut |segment| {
                        builder.cubic_bezier_to(segment.ctrl1, segment.ctrl2, segment.to);
                    });

                    builder.close();
                } else {
                    // Path: InnerStart -> OuterStart -> OuterArc -> InnerEnd -> InnerBackArc -> Close

                    let inner_start = center + Vector::new(start_cos, start_sin) * s_inner;
                    builder.begin(inner_start);

                    let outer_start = center + Vector::new(start_cos, start_sin) * s_outer;
                    builder.line_to(outer_start);

                    let outer_arc = lyon::geom::Arc {
                        center,
                        radii: Vector::new(s_outer, s_outer),
                        start_angle: Angle::radians(self.start_angle),
                        sweep_angle: sweep_a,
                        x_rotation: Angle::radians(0.0),
                    };
                    outer_arc.for_each_cubic_bezier(&mut |segment| {
                        builder.cubic_bezier_to(segment.ctrl1, segment.ctrl2, segment.to);
                    });

                    let inner_end = center + Vector::new(end_cos, end_sin) * s_inner;
                    builder.line_to(inner_end);

                    let inner_arc = lyon::geom::Arc {
                        center,
                        radii: Vector::new(s_inner, s_inner),
                        start_angle: Angle::radians(self.end_angle),
                        sweep_angle: Angle::radians(self.start_angle - self.end_angle),
                        x_rotation: Angle::radians(0.0),
                    };
                    inner_arc.for_each_cubic_bezier(&mut |segment| {
                        builder.cubic_bezier_to(segment.ctrl1, segment.ctrl2, segment.to);
                    });

                    builder.close();
                }
            }

            tess.stroke_path(buffer, builder.build().iter(), stroke, width, 0.1);
        }
    }

    // --- Helpers ---

    fn resolve_length(&self, transform: &Transform<D, D, f32>, len: Length<D>) -> f32 {
        match len {
            Length::Screen(px) => px,
            Length::Plot(units) => {
                // Calculate scale for X (Horizontal)
                let p0_x = transform.x_to_screen(&D::zero());
                let p1_x = transform.x_to_screen(&units);
                let size_x = (p1_x - p0_x).abs();

                // Calculate scale for Y (Vertical)
                let p0_y = transform.y_to_screen(&D::zero());
                let p1_y = transform.y_to_screen(&units);
                let size_y = (p1_y - p0_y).abs();

                // KEY CHANGE: Use the minimum of X/Y scales.
                // This constrains the geometry to the tightest dimension,
                // ensuring perfect circles regardless of aspect ratio.
                size_x.min(size_y)
            }
        }
    }

    /// Generates a Triangle Strip to fill the arc sector.
    fn add_solid_arc_strip(
        &self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        r_inner: f32,
        r_outer: f32,
        start_angle: f32,
        end_angle: f32,
        color: Color,
    ) {
        let packed_color = pack(color);
        let sweep = (end_angle - start_angle).abs();

        let arc_len = sweep * r_outer;
        let segments = (arc_len / 5.0).max(4.0).min(128.0) as usize;

        let step = sweep / segments as f32;
        let dir = if end_angle > start_angle { 1.0 } else { -1.0 };

        let mut vertices = Vec::with_capacity((segments + 1) * 2);
        let mut indices = Vec::with_capacity(segments * 6);

        for i in 0..=segments {
            let theta = start_angle + (i as f32 * step * dir);
            let (sin, cos) = theta.sin_cos();

            vertices.push(SolidVertex2D {
                position: [cx + cos * r_inner, cy + sin * r_inner],
                color: packed_color,
            });

            vertices.push(SolidVertex2D {
                position: [cx + cos * r_outer, cy + sin * r_outer],
                color: packed_color,
            });
        }

        for i in 0..segments {
            let base = (i * 2) as u32;
            // Triangle 1
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 2);
            // Triangle 2
            indices.push(base + 1);
            indices.push(base + 3);
            indices.push(base + 2);
        }

        buffer.add(&indices, &vertices);
    }
}
