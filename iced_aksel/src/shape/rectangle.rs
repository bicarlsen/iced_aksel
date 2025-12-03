use crate::{
    Length, Shape, Stroke, StrokeStyle,
    plot::{self},
    render::{MeshBuffer, Tessellators},
};
use aksel::{Float, PlotPoint, Transform, scale};
use iced::{
    Color,
    advanced::graphics::{color::pack, mesh::SolidVertex2D},
};
use lyon_tessellation::math::Point;

/// A rectangle shape that can be drawn on a chart.
///
/// `Rectangle` supports both **Chart Data** coordinates (scaling with zoom) and
/// **Screen** coordinates (fixed pixels). It is designed for high-performance rendering.
///
/// # Example
/// ```rust,no_run
/// use aksel::PlotPoint;
/// use my_crate::{Rectangle, Length, Stroke, StrokeStyle};
/// use iced::Color;
///
/// // A rectangle defined in plot coordinates (e.g., a data region)
/// let rect = Rectangle::new(
///     PlotPoint::new(10.0, 20.0),
///     Length::Plot(5.0),  // 5 units wide
///     Length::Plot(2.0),  // 2 units high
/// )
/// .fill(Color::from_rgb(0.1, 0.2, 0.9))
/// .stroke(Stroke {
///     fill: Color::BLACK,
///     thickness: Length::Screen(2.0), // Fixed 2px border
///     style: StrokeStyle::Solid,
/// });
/// ```
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
        // We now request the 'tess' (Tessellators) from the context
        ctx.render_mesh(move |transform, buffer, tess| {
            self.tessellate(transform, buffer, tess);
        })
    }
}

impl<D: Float> Rectangle<D> {
    // =========================================================================
    //  Constructors
    // =========================================================================

    /// Creates a new `Rectangle` defined by its center point and dimensions.
    ///
    /// By default, the rectangle is invisible (no fill, no stroke).
    /// Use [`.fill()`](Self::fill) and [`.stroke()`](Self::stroke) to define its appearance.
    ///
    /// # Arguments
    /// * `center` - The center position in chart coordinates.
    /// * `width` - The width of the rectangle (Screen pixels or Plot units).
    /// * `height` - The height of the rectangle (Screen pixels or Plot units).
    pub const fn new(center: PlotPoint<D>, width: Length<D>, height: Length<D>) -> Self {
        Self {
            center,
            width,
            height,
            fill: None,
            stroke: None,
        }
    }

    /// Creates a new `Rectangle` defined by two opposing corner points.
    ///
    /// This is useful for defining regions or selection boxes where you know the
    /// min/max coordinates but not the center.
    ///
    /// This method automatically calculates the `center`, `width`, and `height`
    /// in `Length::Plot` units.
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
    //  Builder Methods
    // =========================================================================

    /// Sets the fill color of the rectangle.
    ///
    /// If a stroke is also set, the fill will automatically "tuck" under the stroke
    /// to prevents anti-aliasing bleed artifacts.
    #[inline]
    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets the stroke (border) of the rectangle.
    ///
    /// The stroke is rendered **inside** the defined dimensions (`Inner` alignment).
    #[inline]
    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Removes the fill, making the body of the rectangle transparent.
    #[inline]
    pub const fn no_fill(mut self) -> Self {
        self.fill = None;
        self
    }

    // =========================================================================
    //  Hybrid Tessellation Logic (Manual + Lyon)
    // =========================================================================

    /// Calculates the screen-space boundaries (min_x, max_x, min_y, max_y).
    fn resolve_bounds(&self, transform: &Transform<D, D, f32>) -> (f32, f32, f32, f32) {
        let half_const = D::from(0.5).unwrap();

        // Resolve Width
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

        // Resolve Height
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

    fn tessellate(
        self,
        transform: &Transform<D, D, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        let (x_min, x_max, y_min, y_max) = self.resolve_bounds(transform);
        let width = x_max - x_min;
        let height = y_max - y_min;

        // 1. Resolve Stroke Thickness (if any)
        // We calculate precise X and Y thickness to support non-uniform scaling
        // in the Manual implementation.
        let maybe_stroke_data = if let Some(stroke) = &self.stroke {
            let (th_x, th_y) = match stroke.thickness {
                Length::Screen(px) => (px, px),
                Length::Plot(units) => (
                    (transform.x_to_screen(&units) - transform.x_to_screen(&D::zero())).abs(),
                    (transform.y_to_screen(&units) - transform.y_to_screen(&D::zero())).abs(),
                ),
            };

            // Optimization: Skip invisible strokes
            if th_x < 0.1 && th_y < 0.1 {
                None
            } else {
                Some((th_x, th_y, stroke))
            }
        } else {
            None
        };

        // 2. Rule 2: Geometric Stability (Consumption Check)
        // If the stroke consumes the shape, we render a single solid block.
        // This avoids overdraw and artifacts.
        let is_consumed = if let Some((th_x, th_y, _)) = maybe_stroke_data {
            th_x >= width * 0.5 || th_y >= height * 0.5
        } else {
            false
        };

        // FAST PATH: Shape is fully consumed by stroke
        if is_consumed {
            if let Some((_, _, stroke)) = maybe_stroke_data {
                // Use Manual Quad for maximum speed
                self.add_solid_quad(buffer, x_min, x_max, y_min, y_max, stroke.fill);
            }
            return;
        }

        // 3. Render Fill (Manual Optimized)
        if let Some(fill_color) = self.fill {
            // Rule 3: Anti-Aliasing Polish (Bleed)
            // If a stroke exists, deflate the fill by 0.5px to tuck it under the stroke.
            let d = if maybe_stroke_data.is_some() && width > 1.0 && height > 1.0 {
                0.5
            } else {
                0.0
            };

            self.add_solid_quad(
                buffer,
                x_min + d,
                x_max - d,
                y_min + d,
                y_max - d,
                fill_color,
            );
        }

        // 4. Render Stroke (Hybrid: Manual or Lyon)
        if let Some((th_x, th_y, stroke)) = maybe_stroke_data {
            match stroke.style {
                StrokeStyle::Solid => {
                    // MANUAL PATH:
                    // Much faster than tessellation. Generates exactly 8 vertices.
                    // Supports non-uniform thickness (th_x != th_y).
                    self.add_manual_stroke(
                        buffer,
                        x_min,
                        x_max,
                        y_min,
                        y_max,
                        th_x,
                        th_y,
                        stroke.fill,
                    );
                }
                StrokeStyle::Dashed | StrokeStyle::Dotted => {
                    // LYON PATH:
                    // Necessary for complex dashes.
                    // Limitation: Lyon assumes uniform thickness, so we average th_x/th_y.
                    let thickness = (th_x + th_y) / 2.0;

                    // Rule 1: Inner Stroke Alignment
                    // Deflate path by thickness/2 so the stroke sits inside bounds.
                    let offset = thickness / 2.0;

                    let points = vec![
                        Point::new(x_min + offset, y_min + offset),
                        Point::new(x_max - offset, y_min + offset),
                        Point::new(x_max - offset, y_max - offset),
                        Point::new(x_min + offset, y_max - offset),
                        Point::new(x_min + offset, y_min + offset), // Explicit close
                    ];

                    tess.stroke_polyline(
                        buffer, points, stroke, thickness, true, // close_path
                    );
                }
            }
        }
    }

    // --- Helpers for Manual Tessellation ---

    /// Adds a simple solid rectangle (2 triangles) to the buffer.
    #[inline(always)]
    fn add_solid_quad(
        &self,
        buffer: &mut MeshBuffer,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        color: Color,
    ) {
        let color = pack(color);
        buffer.add(
            &[0, 1, 2, 1, 2, 3],
            &[
                SolidVertex2D {
                    position: [x_min, y_min],
                    color,
                }, // BL
                SolidVertex2D {
                    position: [x_max, y_min],
                    color,
                }, // BR
                SolidVertex2D {
                    position: [x_min, y_max],
                    color,
                }, // TL
                SolidVertex2D {
                    position: [x_max, y_max],
                    color,
                }, // TR
            ],
        );
    }

    /// Adds a hollow rectangular frame (inner stroke) using 8 vertices.
    #[inline(always)]
    fn add_manual_stroke(
        &self,
        buffer: &mut MeshBuffer,
        x_min: f32,
        x_max: f32,
        y_min: f32,
        y_max: f32,
        th_x: f32,
        th_y: f32,
        color: Color,
    ) {
        let color = pack(color);

        // Calculate inner bounds
        // Note: We don't need to check for inversion here because
        // the 'consumption check' (is_consumed) in tessellate() guarantees
        // thickness is small enough to fit.
        let ix_min = x_min + th_x;
        let ix_max = x_max - th_x;
        let iy_min = y_min + th_y;
        let iy_max = y_max - th_y;

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
