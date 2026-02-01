use iced_core::{Point, Rectangle, Size};
use lyon::math::point as lpoint;
use lyon_path::{EndpointId, PathEvent};
use std::f32::consts::PI; // Helper to convert to Lyon points

/// The atomic drawing commands supported by your engine.
#[derive(Clone, Copy, Debug)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),         // Control, End
    CubicTo(Point, Point, Point), // Control1, Control2, End
    Close,
}

/// A simplified, engine-agnostic buffer for geometry.
///
/// This is the "Intermediate Representation" (IR). primitives write to this,
/// and Renderers (Mesh/Skia) consume from this.
#[derive(Clone, Debug, Default)]
pub struct GeometryBuffer {
    commands: Vec<PathCommand>,
    // Optional: cache bounds or convexity here if needed for optimization
}

impl GeometryBuffer {
    /// Creates a new, empty geometry buffer.
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(64), // Pre-alloc reasonable size
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    // --- Builder API (The "Writer" Interface) ---

    pub fn move_to(&mut self, p: Point) {
        self.commands.push(PathCommand::MoveTo(p));
    }

    pub fn line_to(&mut self, p: Point) {
        self.commands.push(PathCommand::LineTo(p));
    }

    pub fn quadratic_bezier_to(&mut self, control: Point, end: Point) {
        self.commands.push(PathCommand::QuadTo(control, end));
    }

    pub fn cubic_bezier_to(&mut self, control1: Point, control2: Point, end: Point) {
        self.commands
            .push(PathCommand::CubicTo(control1, control2, end));
    }

    pub fn close(&mut self) {
        self.commands.push(PathCommand::Close);
    }

    // --- Granular Helpers (Your "Granularity" Request) ---

    /// fast-paths a rectangle into the buffer
    pub fn rectangle(&mut self, rect: Rectangle) {
        let x = rect.x;
        let y = rect.y;
        let w = rect.width;
        let h = rect.height;

        // We manually unroll this so it's just 5 commands
        self.move_to(Point::new(x, y));
        self.line_to(Point::new(x + w, y));
        self.line_to(Point::new(x + w, y + h));
        self.line_to(Point::new(x, y + h));
        self.close();
    }

    /// Adds an ellipse to the buffer using 4 cubic Bezier curves.
    pub fn ellipse(&mut self, center: Point, radius_x: f32, radius_y: f32) {
        // The magic constant for 4-spline circle approximation
        const K: f32 = 0.5522847498;

        let kx = radius_x * K;
        let ky = radius_y * K;

        // 4 Quadrants
        // Q1: Right -> Bottom
        let p0 = Point::new(center.x + radius_x, center.y);
        let p1 = Point::new(center.x, center.y + radius_y);
        let p2 = Point::new(center.x - radius_x, center.y);
        let p3 = Point::new(center.x, center.y - radius_y);

        self.move_to(p0);

        // Curve 1: Right to Bottom
        self.cubic_bezier_to(Point::new(p0.x, p0.y + ky), Point::new(p1.x + kx, p1.y), p1);

        // Curve 2: Bottom to Left
        self.cubic_bezier_to(Point::new(p1.x - kx, p1.y), Point::new(p2.x, p2.y + ky), p2);

        // Curve 3: Left to Top
        self.cubic_bezier_to(Point::new(p2.x, p2.y - ky), Point::new(p3.x - kx, p3.y), p3);

        // Curve 4: Top to Right
        self.cubic_bezier_to(Point::new(p3.x + kx, p3.y), Point::new(p0.x, p0.y - ky), p0);

        self.close();
    }

    /// Adds a regular polygon (Triangle, Hexagon, etc.)
    pub fn polygon(&mut self, center: Point, radius: f32, vertices: u16, rotation_degrees: f32) {
        if vertices < 3 {
            return;
        }

        // Convert rotation to radians (starting pointing up by default usually implies -90 deg offset,
        // but mathematical standard is 0 deg = right. We'll assume standard).
        let rot_rad = rotation_degrees.to_radians();
        let step = 2.0 * PI / vertices as f32;

        for i in 0..vertices {
            let theta = rot_rad + step * i as f32;
            let p = Point::new(
                center.x + radius * theta.cos(),
                center.y + radius * theta.sin(),
            );

            if i == 0 {
                self.move_to(p);
            } else {
                self.line_to(p);
            }
        }
        self.close();
    }

    /// Adds a generic polyline (open or closed).
    pub fn polyline(&mut self, points: &[Point], close: bool) {
        if points.is_empty() {
            return;
        }
        self.move_to(points[0]);
        for p in &points[1..] {
            self.line_to(*p);
        }
        if close {
            self.close();
        }
    }

    /// Adds a Catmull-Rom spline.
    /// Converts the points into a series of Cubic Bezier curves.
    pub fn catmull_rom_spline(&mut self, points: &[Point], tension: f32) {
        if points.len() < 2 {
            return;
        }

        // 1. Move to start
        self.move_to(points[0]);

        // 2. Iterate segments
        for i in 0..points.len() - 1 {
            let p0 = if i == 0 { points[0] } else { points[i - 1] };
            let p1 = points[i];
            let p2 = points[i + 1];
            let p3 = if i + 2 < points.len() {
                points[i + 2]
            } else {
                p2
            };

            // Calculate Control Points
            // Tangent M1 = (P2 - P0) * tension
            // Tangent M2 = (P3 - P1) * tension
            // C1 = P1 + M1 / 6.0 (approx smoothing factor)
            // C2 = P2 - M2 / 6.0

            // Note: 'tension' in standard APIs often implies 0.0=straight, 1.0=curvy.
            // A common scaler for Catmull-Rom -> Bezier is 1.0/6.0 for standard smoothness.
            // We'll use tension as a direct multiplier for the tangent magnitude.
            let scaler = tension / 6.0;

            let c1 = Point::new(p1.x + (p2.x - p0.x) * scaler, p1.y + (p2.y - p0.y) * scaler);

            let c2 = Point::new(p2.x - (p3.x - p1.x) * scaler, p2.y - (p3.y - p1.y) * scaler);

            self.cubic_bezier_to(c1, c2, p2);
        }
    }

    /// Adds an circular arc approximated by Bezier curves.
    pub fn arc(&mut self, center: Point, radius: f32, start_angle: f32, end_angle: f32) {
        // Ensure we iterate in the direction of the sweep
        let sweep = end_angle - start_angle;
        let abs_sweep = sweep.abs();

        // Split into segments of at most 90 degrees (PI/2) to minimize error
        let max_seg = PI / 2.0;
        let n_segments = (abs_sweep / max_seg).ceil() as usize;
        let step = sweep / n_segments as f32;

        // Move to start of arc
        let start_p = Point::new(
            center.x + radius * start_angle.cos(),
            center.y + radius * start_angle.sin(),
        );
        self.move_to(start_p);

        for i in 0..n_segments {
            let theta0 = start_angle + step * i as f32;
            let theta1 = start_angle + step * (i + 1) as f32;

            // Control point distance factor for this segment span
            // k = (4/3) * tan(delta / 4)
            let delta = step;
            let k = (4.0 / 3.0) * (delta / 4.0).tan();

            // Bezier Points calculation
            // P0 is implicitly current point
            // P3 is destination
            let p3 = Point::new(
                center.x + radius * theta1.cos(),
                center.y + radius * theta1.sin(),
            );

            // Derivatives (tangents) rotated by 90 degrees
            // C1 = P0 + k * tangent(theta0)
            let c1 = Point::new(
                (center.x + radius * theta0.cos()) - k * radius * theta0.sin(),
                (center.y + radius * theta0.sin()) + k * radius * theta0.cos(),
            );

            // C2 = P3 - k * tangent(theta1)
            let c2 = Point::new(
                p3.x + k * radius * theta1.sin(), // Note signs flip for incoming tangent
                p3.y - k * radius * theta1.cos(),
            );

            self.cubic_bezier_to(c1, c2, p3);
        }
    }

    // --- Exporters (The "Reader" Interface) ---

    /// Returns an iterator compatible with the Lyon Tessellator.
    /// Used by `MeshBatcher`.
    pub fn lyon_iter(&self) -> LyonAdapter<'_> {
        LyonAdapter {
            iter: self.commands.iter(),
            current: lpoint(0.0, 0.0),
            first: lpoint(0.0, 0.0),
        }
    }

    /// Populates an Iced/Skia Builder with these commands.
    /// Used by `PathBatcher`.
    pub fn populate_iced(&self, builder: &mut iced_graphics::geometry::path::Builder) {
        for cmd in &self.commands {
            match cmd {
                PathCommand::MoveTo(p) => builder.move_to(*p),
                PathCommand::LineTo(p) => builder.line_to(*p),
                PathCommand::QuadTo(c, e) => builder.quadratic_curve_to(*c, *e),
                PathCommand::CubicTo(c1, c2, e) => builder.bezier_curve_to(*c1, *c2, *e),
                PathCommand::Close => builder.close(),
            }
        }
    }
}

// --- The Lyon Adapter ---
// This magic struct makes your Vec<Command> look like a stream of Lyone events.

pub struct LyonAdapter<'a> {
    iter: std::slice::Iter<'a, PathCommand>,
    current: lyon::math::Point,
    first: lyon::math::Point,
}

impl<'a> Iterator for LyonAdapter<'a> {
    type Item = PathEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let cmd = self.iter.next()?;

        match cmd {
            PathCommand::MoveTo(p) => {
                let pt = lpoint(p.x, p.y);
                self.current = pt;
                self.first = pt;
                Some(PathEvent::Begin { at: pt })
            }
            PathCommand::LineTo(p) => {
                let pt = lpoint(p.x, p.y);
                let from = self.current;
                self.current = pt;
                Some(PathEvent::Line { from, to: pt })
            }
            PathCommand::QuadTo(c, e) => {
                let ctrl = lpoint(c.x, c.y);
                let to = lpoint(e.x, e.y);
                let from = self.current;
                self.current = to;
                Some(PathEvent::Quadratic { from, ctrl, to })
            }
            PathCommand::CubicTo(c1, c2, e) => {
                let ctrl1 = lpoint(c1.x, c1.y);
                let ctrl2 = lpoint(c2.x, c2.y);
                let to = lpoint(e.x, e.y);
                let from = self.current;
                self.current = to;
                Some(PathEvent::Cubic {
                    from,
                    ctrl1,
                    ctrl2,
                    to,
                })
            }
            PathCommand::Close => {
                let last = self.current;
                let first = self.first;
                self.current = first;
                // EndpointId(0) is a dummy ID required by Lyon, usually ignored
                Some(PathEvent::End {
                    last,
                    first,
                    close: true,
                })
            }
        }
    }
}
