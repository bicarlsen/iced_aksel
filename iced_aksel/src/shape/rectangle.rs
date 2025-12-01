use aksel::{Float, PlotPoint, Transform, scale};
use iced::{
    Color,
    advanced::graphics::{color::pack, mesh::SolidVertex2D},
};

use crate::{Length, Shape, Stroke, plot, render::MeshBuffer};

#[derive(Debug, Clone)]
pub struct Rectangle<D> {
    center: PlotPoint<D>,
    width: Length<D>,
    height: Length<D>,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Rectangle<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, _tess| {
            self.manual_tessellation(transform, buffer);
        })
    }
}

impl<D: Float> Rectangle<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    /// Creates a new Rectangle defined by its center point and dimensions.
    ///
    /// By default, the rectangle has no fill and no stroke (invisible).
    /// Use `.fill()` or `.stroke()` to define its style.
    pub const fn new(center: PlotPoint<D>, width: Length<D>, height: Length<D>) -> Self {
        Self {
            center,
            width,
            height,
            fill: None,
            stroke: None,
        }
    }

    /// Creates a new Rectangle defined by two opposing corner points.
    ///
    /// This automatically calculates the center and dimensions.
    pub fn from_corners(p1: PlotPoint<D>, p2: PlotPoint<D>) -> Self {
        let (x_min, x_max) = scale::util::sorted_pair(p1.x, p2.x);
        let (y_min, y_max) = scale::util::sorted_pair(p1.y, p2.y);

        let two = D::one() + D::one();

        Self {
            center: PlotPoint {
                x: (x_min + x_max) / two,
                y: (y_min + y_max) / two,
            },
            width: Length::Plot(x_max - x_min),
            height: Length::Plot(y_max - y_min),
            fill: None,
            stroke: None,
        }
    }

    // =========================================================================
    //  Builder Methods (Chainable)
    // =========================================================================

    /// Sets the fill color of the rectangle.
    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke of the rectangle.
    #[inline]
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Clears the fill (making the body transparent).
    #[inline]
    pub const fn no_fill(mut self) -> Self {
        self.fill = None;
        self
    }

    // =========================================================================
    //  Rendering Logic (Optimized)
    // =========================================================================

    /// Calculates the screen-space boundaries.
    #[inline(always)]
    fn resolve_bounds(&self, transform: &Transform<D, D, f32>) -> (f32, f32, f32, f32) {
        // Safety: Unwrapping is okay here, as 0.5 should always resolve to a `Float`
        let half_const = D::from(0.5).unwrap();

        // --- X AXIS ---
        let (x_min, x_max) = match &self.width {
            Length::Screen(px) => {
                let c = transform.x_to_screen(&self.center.x);
                let half = px * 0.5;
                (c - half, c + half)
            }
            Length::Plot(width) => {
                let half_w = *width * half_const;
                let p1 = transform.x_to_screen(&(self.center.x - half_w));
                let p2 = transform.x_to_screen(&(self.center.x + half_w));
                if p1 < p2 { (p1, p2) } else { (p2, p1) }
            }
        };

        // --- Y AXIS ---
        let (y_min, y_max) = match &self.height {
            Length::Screen(px) => {
                let c = transform.y_to_screen(&self.center.y);
                let half = px * 0.5;
                (c - half, c + half)
            }
            Length::Plot(height) => {
                let half_h = *height * half_const;
                let p1 = transform.y_to_screen(&(self.center.y - half_h));
                let p2 = transform.y_to_screen(&(self.center.y + half_h));
                if p1 < p2 { (p1, p2) } else { (p2, p1) }
            }
        };

        (x_min, x_max, y_min, y_max)
    }

    fn manual_tessellation(self, transform: &Transform<D, D, f32>, buffer: &mut MeshBuffer) {
        let (x_min, x_max, y_min, y_max) = self.resolve_bounds(transform);

        // Render Fill
        if let Some(fill_color) = self.fill {
            let color = pack(fill_color);

            // CORRECTION 1: Removed 'offset = buffer.offset()'
            // CORRECTION 2: Indices start at 0.
            buffer.add(
                &[
                    0, 1, 2, // Triangle 1
                    1, 2, 3, // Triangle 2
                ],
                &[
                    SolidVertex2D {
                        position: [x_min, y_min],
                        color,
                    }, // 0: BL
                    SolidVertex2D {
                        position: [x_max, y_min],
                        color,
                    }, // 1: BR
                    SolidVertex2D {
                        position: [x_min, y_max],
                        color,
                    }, // 2: TL
                    SolidVertex2D {
                        position: [x_max, y_max],
                        color,
                    }, // 3: TR
                ],
            );
        }

        // Render Stroke
        if let Some(stroke) = &self.stroke {
            self.render_stroke_manual(transform, buffer, stroke, (x_min, x_max), (y_min, y_max));
        }
    }

    #[inline(always)]
    fn render_stroke_manual(
        &self,
        transform: &Transform<D, D, f32>,
        buffer: &mut MeshBuffer,
        stroke: &Stroke<D>,
        x_min_max: (f32, f32),
        y_min_max: (f32, f32),
    ) {
        // ... (Thickness calculation remains the same) ...
        let (th_x, th_y) = match stroke.thickness {
            Length::Screen(px) => (px, px),
            Length::Plot(units) => (
                (transform.x_to_screen(&units) - transform.x_to_screen(&D::zero())).abs(),
                (transform.y_to_screen(&units) - transform.y_to_screen(&D::zero())).abs(),
            ),
        };

        if th_x < 0.1 && th_y < 0.1 {
            return;
        }

        let (x_min, x_max) = x_min_max;
        let (y_min, y_max) = y_min_max;

        let width = x_max - x_min;
        let height = y_max - y_min;
        let inset_x = th_x.min(width * 0.5);
        let inset_y = th_y.min(height * 0.5);

        let (ix_min, ix_max) = (x_min + inset_x, x_max - inset_x);
        let (iy_min, iy_max) = (y_min + inset_y, y_max - inset_y);

        let color = pack(stroke.fill);

        // CORRECTION: Indices are now relative to this specific batch (0..7)
        // No 'offset' variable needed.
        buffer.add(
            &[
                0, 1, 4, 1, 4, 5, // Bottom Edge
                1, 2, 5, 2, 5, 6, // Right Edge
                2, 3, 6, 3, 6, 7, // Top Edge
                3, 0, 7, 0, 7, 4, // Left Edge
            ],
            &[
                // Outer Ring (0-3)
                SolidVertex2D {
                    position: [x_min, y_min],
                    color,
                },
                SolidVertex2D {
                    position: [x_max, y_min],
                    color,
                },
                SolidVertex2D {
                    position: [x_max, y_max],
                    color,
                },
                SolidVertex2D {
                    position: [x_min, y_max],
                    color,
                },
                // Inner Ring (4-7)
                SolidVertex2D {
                    position: [ix_min, iy_min],
                    color,
                },
                SolidVertex2D {
                    position: [ix_max, iy_min],
                    color,
                },
                SolidVertex2D {
                    position: [ix_max, iy_max],
                    color,
                },
                SolidVertex2D {
                    position: [ix_min, iy_max],
                    color,
                },
            ],
        );
    }
}
