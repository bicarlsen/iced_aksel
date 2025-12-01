use aksel::{Float, PlotPoint, Transform};
use iced::Color;
use lyon::geom::Arc;
use lyon::math::{Angle, Vector, point};
use lyon::path::Path;
use lyon::path::builder::PathBuilder;
use lyon_path::Attributes;

use crate::{
    Length, Shape, Stroke, plot,
    render::{MeshBuffer, Tessellators},
};

#[derive(Debug, Clone)]
pub struct Sector<D> {
    center: PlotPoint<D>,
    radius: Length<D>,
    inner_radius: Length<D>,
    start_angle: f32,
    end_angle: f32,
    fill: Option<Color>,
    stroke: Option<Stroke<D>>,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Sector<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_mesh(move |transform, buffer, tess| {
            self.lyon_tessellation(transform, buffer, tess);
        })
    }
}

impl<D: Float> Sector<D> {
    pub fn new(center: PlotPoint<D>, radius: Length<D>, start_deg: f32, end_deg: f32) -> Self {
        let offset = -90.0_f32;
        Self {
            center,
            radius,
            inner_radius: Length::Screen(0.0),
            start_angle: (start_deg + offset).to_radians(),
            end_angle: (end_deg + offset).to_radians(),
            fill: None,
            stroke: None,
        }
    }

    pub const fn inner_radius(mut self, l: Length<D>) -> Self {
        self.inner_radius = l;
        self
    }

    pub const fn fill(mut self, c: Color) -> Self {
        self.fill = Some(c);
        self
    }

    pub const fn stroke(mut self, s: Stroke<D>) -> Self {
        self.stroke = Some(s);
        self
    }

    fn lyon_tessellation(
        self,
        transform: &Transform<D, D, f32>,
        buffer: &mut MeshBuffer,
        tess: &mut Tessellators,
    ) {
        let cx = transform.x_to_screen(&self.center.x);
        let cy = transform.y_to_screen(&self.center.y);
        let center = point(cx, cy);
        let outer_r = self.resolve_len(&self.radius, transform);
        let inner_r = self.resolve_len(&self.inner_radius, transform);

        if outer_r < 0.5 {
            return;
        }

        let start = Angle::radians(self.start_angle);
        let sweep = Angle::radians(self.end_angle - self.start_angle);
        let is_full = sweep.radians.abs() >= 2.0f32.mul_add(std::f32::consts::PI, -0.001);

        let stroke_width = self
            .stroke
            .as_ref()
            .map_or(0.0, |stroke| self.resolve_len(&stroke.thickness, transform));

        // SAFETY 1: Radial Consumption
        // If the stroke is so thick it consumes the shape, render silhouette.
        let is_radially_consumed = if inner_r > 0.5 {
            stroke_width >= (outer_r - inner_r)
        } else {
            stroke_width >= outer_r
        };

        if is_radially_consumed {
            self.render_silhouette(
                tess,
                buffer,
                center,
                outer_r,
                inner_r,
                start,
                sweep,
                is_full,
                self.stroke.as_ref().map(|s| s.fill),
            );
            return;
        }

        // --- 1. FILL ---
        if let Some(color) = self.fill {
            let mut builder = Path::builder();
            let adjust = if stroke_width > 0.0 {
                stroke_width.min(0.5)
            } else {
                0.0
            };

            let f_out = outer_r - adjust;
            let f_in = if inner_r > 0.5 { inner_r + adjust } else { 0.0 };

            if f_out > f_in {
                self.build_sector_path(&mut builder, center, f_out, f_in, start, sweep, is_full);
                tess.fill_path(buffer, builder.build().iter(), color, 0.1);
            }
        }

        // --- 2. STROKE ---
        if let Some(stroke) = &self.stroke {
            let half_w = stroke_width / 2.0;
            let s_out = outer_r - half_w;
            let s_in = if inner_r > 0.5 { inner_r + half_w } else { 0.0 };

            if s_out < 0.1 {
                return;
            }

            // Angular Inset Logic
            let da_out = Angle::radians((half_w / s_out).min(1.0).asin());
            let da_in = if s_in > 1.0 {
                Angle::radians((half_w / s_in).min(1.0).asin())
            } else {
                Angle::radians(0.0)
            };

            let start_out = start + da_out;
            let sweep_out = sweep - da_out * 2.0;

            let start_in = start + da_in;
            let sweep_in = sweep - da_in * 2.0;

            // SAFETY 2: Angular Consumption
            if !is_full && sweep_out.radians <= 0.0 {
                self.render_silhouette(
                    tess,
                    buffer,
                    center,
                    outer_r,
                    inner_r,
                    start,
                    sweep,
                    is_full,
                    Some(stroke.fill),
                );
                return;
            }

            let mut builder = Path::builder();

            if is_full {
                builder.add_circle(center, s_out, lyon::path::Winding::Positive);
                if inner_r >= 0.5 && s_in < s_out {
                    builder.add_circle(center, s_in, lyon::path::Winding::Positive);
                }
            } else if inner_r < 0.5 {
                // Pie Slice Logic
                let half_sweep = sweep.radians / 2.0;
                let sin_half = half_sweep.sin().abs().max(0.001);
                let dist_shift = half_w / sin_half;

                // SAFETY 3: Geometric Inversion Check (Fix for Star Artifact)
                // If the shift required for the stroke corners pushes the center
                // point BEYOND the outer arc, the geometry is inverted.
                if dist_shift >= s_out {
                    self.render_silhouette(
                        tess,
                        buffer,
                        center,
                        outer_r,
                        inner_r,
                        start,
                        sweep,
                        is_full,
                        Some(stroke.fill),
                    );
                    return;
                }

                let mid_angle = start + Angle::radians(half_sweep);
                let shift_vec =
                    Vector::new(mid_angle.radians.cos(), mid_angle.radians.sin()) * dist_shift;
                let virtual_center = center + shift_vec;

                let outer = Arc {
                    center,
                    radii: Vector::new(s_out, s_out),
                    start_angle: start_out,
                    sweep_angle: sweep_out,
                    x_rotation: Angle::radians(0.0),
                };

                builder.begin(virtual_center);
                builder.line_to(outer.from());
                outer.for_each_cubic_bezier(&mut |s| {
                    builder.cubic_bezier_to(s.ctrl1, s.ctrl2, s.to);
                });
                builder.close();
            } else if sweep_in.radians <= 0.0 {
                // Donut Logic
                // Inner ring consumed
                let half_sweep = sweep.radians / 2.0;
                let sin_half = half_sweep.sin().abs().max(0.001);
                let dist_shift = half_w / sin_half;

                // Same Safety Check for partial donuts
                if dist_shift >= s_out {
                    self.render_silhouette(
                        tess,
                        buffer,
                        center,
                        outer_r,
                        inner_r,
                        start,
                        sweep,
                        is_full,
                        Some(stroke.fill),
                    );
                    return;
                }

                let mid_angle = start + Angle::radians(half_sweep);
                let shift_vec =
                    Vector::new(mid_angle.radians.cos(), mid_angle.radians.sin()) * dist_shift;
                let virtual_center = center + shift_vec;

                let outer = Arc {
                    center,
                    radii: Vector::new(s_out, s_out),
                    start_angle: start_out,
                    sweep_angle: sweep_out,
                    x_rotation: Angle::radians(0.0),
                };

                builder.begin(virtual_center);
                builder.line_to(outer.from());
                outer.for_each_cubic_bezier(&mut |s| {
                    builder.cubic_bezier_to(s.ctrl1, s.ctrl2, s.to);
                });
                builder.close();
            } else {
                // Standard Donut
                let outer = Arc {
                    center,
                    radii: Vector::new(s_out, s_out),
                    start_angle: start_out,
                    sweep_angle: sweep_out,
                    x_rotation: Angle::radians(0.0),
                };
                let inner = Arc {
                    center,
                    radii: Vector::new(s_in, s_in),
                    start_angle: start_in + sweep_in,
                    sweep_angle: -sweep_in,
                    x_rotation: Angle::radians(0.0),
                };

                builder.begin(outer.from());
                outer.for_each_cubic_bezier(&mut |s| {
                    builder.cubic_bezier_to(s.ctrl1, s.ctrl2, s.to);
                });
                builder.line_to(inner.from());
                inner.for_each_cubic_bezier(&mut |s| {
                    builder.cubic_bezier_to(s.ctrl1, s.ctrl2, s.to);
                });
                builder.close();
            }
            tess.stroke_path(buffer, builder.build().iter(), stroke, stroke_width, 0.1);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_silhouette(
        &self,
        tess: &mut Tessellators,
        buffer: &mut MeshBuffer,
        center: lyon::math::Point,
        r_out: f32,
        r_in: f32,
        start: Angle,
        sweep: Angle,
        is_full: bool,
        color: Option<Color>,
    ) {
        if let Some(c) = color {
            let mut builder = Path::builder();
            self.build_sector_path(&mut builder, center, r_out, r_in, start, sweep, is_full);
            tess.fill_path(buffer, builder.build().iter(), c, 0.1);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_sector_path<B: PathBuilder>(
        &self,
        builder: &mut B,
        center: lyon::math::Point,
        r_out: f32,
        r_in: f32,
        start: Angle,
        sweep: Angle,
        is_full: bool,
    ) {
        let outer = Arc {
            center,
            radii: Vector::new(r_out, r_out),
            start_angle: start,
            sweep_angle: sweep,
            x_rotation: Angle::radians(0.0),
        };

        if r_in < 0.5 {
            if is_full {
                builder.add_circle(
                    center,
                    r_out,
                    lyon::path::Winding::Positive,
                    Attributes::default(),
                );
            } else {
                builder.begin(center, Attributes::default());
                builder.line_to(outer.from(), Attributes::default());
                outer.for_each_cubic_bezier(&mut |s| {
                    builder.cubic_bezier_to(s.ctrl1, s.ctrl2, s.to, Attributes::default());
                });
                builder.close();
            }
        } else {
            let inner = Arc {
                center,
                radii: Vector::new(r_in, r_in),
                start_angle: start + sweep,
                sweep_angle: -sweep,
                x_rotation: Angle::radians(0.0),
            };
            if is_full {
                builder.add_circle(
                    center,
                    r_out,
                    lyon::path::Winding::Positive,
                    Attributes::default(),
                );
                builder.add_circle(
                    center,
                    r_in,
                    lyon::path::Winding::Negative,
                    Attributes::default(),
                );
            } else {
                builder.begin(outer.from(), Attributes::default());
                outer.for_each_cubic_bezier(&mut |s| {
                    builder.cubic_bezier_to(s.ctrl1, s.ctrl2, s.to, Attributes::default());
                });
                builder.line_to(inner.from(), Attributes::default());
                inner.for_each_cubic_bezier(&mut |s| {
                    builder.cubic_bezier_to(s.ctrl1, s.ctrl2, s.to, Attributes::default());
                });
                builder.close();
            }
        }
    }

    fn resolve_len(&self, len: &Length<D>, transform: &Transform<D, D, f32>) -> f32 {
        match len {
            Length::Screen(v) => *v,
            Length::Plot(v) => (transform.x_to_screen(v) - transform.x_to_screen(&D::zero())).abs(),
        }
    }
}
