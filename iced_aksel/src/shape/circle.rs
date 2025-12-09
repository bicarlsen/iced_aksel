use crate::{
    Length, Shape, Stroke, StrokeStyle,
    plot::{self},
    render::{MeshBuffer, Tessellators},
};
use aksel::{Float, PlotPoint, Transform};
use iced::{
    Color,
    advanced::graphics::{color::pack, mesh::SolidVertex2D},
};
use lyon::geom::Arc;
use lyon::math::{Angle, Point, Vector};

/// A circle shape defined by a center point and a radius.
///
/// # Rendering Strategy (Hybrid Engine)
/// This shape utilizes a dual-path rendering strategy for maximum performance:
/// * **Manual Tessellation (Fast Path):** For solid fills and solid strokes, vertices are
///   generated directly on the CPU using trigonometric lookups (Triangle Fan/Strip). This avoids
///   all memory allocations associated with path builders.
/// * **Lyon Tessellation (Robust Path):** For dashed or dotted strokes, the shape falls back to
///   Lyon's tessellator to handle the complex path events correctly. (significantly slower when rendering thousands)
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

    /// Creates a new `Circle`.
    ///
    /// # Arguments
    /// * `center` - The position of the center in Plot coordinates.
    /// * `radius` - The radius, either in fixed Screen pixels or scalable Plot units.
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

    /// Sets the fill color.
    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke style.
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
        // 1. Resolve Geometry to Screen Space
        let cx = transform.x_to_screen(&self.center.x);
        let cy = transform.y_to_screen(&self.center.y);

        // Calculate Radius in Pixels.
        // We use the MINIMUM of X and Y scales.
        // This ensures the marker remains a perfect circle that fits within its
        // data bounds, even if the aspect ratio is skewed.
        let r = match self.radius {
            Length::Screen(pixels) => pixels,
            Length::Plot(units) => {
                let x0 = transform.x_to_screen(&D::zero());
                let x1 = transform.x_to_screen(&units);
                let dx = (x1 - x0).abs();

                let y0 = transform.y_to_screen(&D::zero());
                let y1 = transform.y_to_screen(&units);
                let dy = (y1 - y0).abs();

                dx.min(dy)
            }
        };

        // Cull sub-pixel circles to save GPU cycles
        if r < 0.5 {
            return;
        }

        // 2. Resolve Stroke Thickness
        let maybe_stroke_data = if let Some(stroke) = &self.stroke {
            let width = match stroke.thickness {
                Length::Screen(w) => w,
                Length::Plot(w) => {
                    let x0 = transform.x_to_screen(&D::zero());
                    let x1 = transform.x_to_screen(&w);
                    let dx = (x1 - x0).abs();

                    let y0 = transform.y_to_screen(&D::zero());
                    let y1 = transform.y_to_screen(&w);
                    let dy = (y1 - y0).abs();

                    dx.min(dy)
                }
            };
            // Optimization: Ignore effectively invisible strokes
            if width < 0.1 {
                None
            } else {
                Some((width, stroke))
            }
        } else {
            None
        };

        // 3. Rule 2: Geometric Stability (Consumption Check)
        // If the stroke is thicker than the radius, the inner hole vanishes.
        // We must switch to a solid fill to prevent geometric inversion artifacts.
        let is_consumed = if let Some((width, _)) = maybe_stroke_data {
            width >= r
        } else {
            false
        };

        // FAST PATH: Shape is consumed. Render as a single solid circle.
        if is_consumed {
            if let Some((_, stroke)) = maybe_stroke_data {
                self.add_solid_circle(buffer, cx, cy, r, stroke.fill);
            }
            return;
        }

        // 4. Render Fill (Manual Optimized)
        if let Some(color) = self.fill {
            // Rule 3: Anti-Aliasing Polish (Bleed Fix)
            // If a stroke exists, we deflate the fill radius slightly (0.5px)
            // to tuck it under the stroke, preventing background bleed-through.
            let fill_r = if maybe_stroke_data.is_some() {
                (r - 0.5).max(0.0)
            } else {
                r
            };

            if fill_r > 0.1 {
                self.add_solid_circle(buffer, cx, cy, fill_r, color);
            }
        }

        // 5. Render Stroke (Hybrid)
        if let Some((width, stroke)) = maybe_stroke_data {
            match stroke.style {
                StrokeStyle::Solid => {
                    // MANUAL PATH (Ring)
                    // Rule 1: Inner Stroke Alignment.
                    // The stroke grows inward from the radius.
                    // Outer Edge = r, Inner Edge = r - width.
                    let inner_r = r - width;
                    let outer_r = r;
                    self.add_solid_ring(buffer, cx, cy, inner_r, outer_r, stroke.fill);
                }
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    // LYON PATH (Dashed)
                    // Rule 1: Inner Stroke Alignment.
                    // Lyon strokes are centered on the path. We offset the path inward by half-width.
                    let stroke_radius = r - (width / 2.0);
                    if stroke_radius > 0.1 {
                        let center = Point::new(cx, cy);
                        let arc = Arc {
                            center,
                            radii: Vector::new(stroke_radius, stroke_radius),
                            start_angle: Angle::radians(0.0),
                            sweep_angle: Angle::radians(std::f32::consts::TAU),
                            x_rotation: Angle::radians(0.0),
                        };

                        tess.stroke_polyline(
                            buffer,
                            arc.flattened(0.2), // Coarse tolerance is fine for dashes
                            stroke,
                            width,
                            true, // Closed path
                        );
                    }
                }
            }
        }
    }

    // --- Manual Tessellation Helpers ---

    /// Generates a **Triangle Fan** for a solid circle.
    ///
    /// This is highly optimized for memory, allocating zero intermediate buffers
    /// and writing directly to the MeshBuffer.
    fn add_solid_circle(&self, buffer: &mut MeshBuffer, cx: f32, cy: f32, r: f32, color: Color) {
        let packed_color = pack(color);

        // Level of Detail (LOD):
        // Scale segment count with radius to maintain smoothness without wasting triangles on dots.
        // Range: 24 segments (small) to 128 segments (large).
        let segments = (r * 2.0).max(24.0).min(128.0) as usize;

        let start_offset = buffer.vertices_count() as u32;
        let mut vertices = Vec::with_capacity(segments + 2); // Center + perimeter
        let mut indices = Vec::with_capacity(segments * 3);

        // 1. Push Center
        vertices.push(SolidVertex2D {
            position: [cx, cy],
            color: packed_color,
        });

        // 2. Generate Perimeter Vertices & Indices
        let step = std::f32::consts::TAU / segments as f32;
        for i in 0..=segments {
            // Only generate N unique vertices (last one wraps to first)
            if i < segments {
                let theta = i as f32 * step;
                let (sin, cos) = theta.sin_cos();
                vertices.push(SolidVertex2D {
                    position: [cx + cos * r, cy + sin * r],
                    color: packed_color,
                });
            }

            // Generate Triangle Fan Indices
            // Center(0) -> Current(i+1) -> Next(i+2 or wrap to 1)
            if i < segments {
                let center_idx = 0;
                let current_idx = (i + 1) as u32;
                let next_idx = if i == segments - 1 {
                    1
                } else {
                    current_idx + 1
                };
                indices.push(center_idx);
                indices.push(current_idx);
                indices.push(next_idx);
            }
        }
        buffer.add(&indices, &vertices);
    }

    /// Generates a **Triangle Strip** for a solid ring (annulus).
    ///
    /// Used for drawing solid strokes efficiently.
    fn add_solid_ring(
        &self,
        buffer: &mut MeshBuffer,
        cx: f32,
        cy: f32,
        r_inner: f32,
        r_outer: f32,
        color: Color,
    ) {
        let packed_color = pack(color);
        let segments = (r_outer * 2.0).max(24.0).min(128.0) as usize;

        let mut vertices = Vec::with_capacity(segments * 2);
        let mut indices = Vec::with_capacity(segments * 6); // 2 triangles per segment
        let step = std::f32::consts::TAU / segments as f32;

        // 1. Generate Vertex Pairs (Inner + Outer)
        for i in 0..segments {
            let theta = i as f32 * step;
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

        // 2. Stitch Quads
        for i in 0..segments {
            let i = i as u32;
            let next_i = (i + 1) % segments as u32;

            // Vertex Layout: [Inner0, Outer0, Inner1, Outer1...]
            let inner_current = i * 2;
            let outer_current = i * 2 + 1;
            let inner_next = next_i * 2;
            let outer_next = next_i * 2 + 1;

            // Triangle 1: InnerCurrent -> OuterCurrent -> OuterNext
            indices.push(inner_current);
            indices.push(outer_current);
            indices.push(outer_next);

            // Triangle 2: InnerCurrent -> OuterNext -> InnerNext
            indices.push(inner_current);
            indices.push(outer_next);
            indices.push(inner_next);
        }
        buffer.add(&indices, &vertices);
    }
}
