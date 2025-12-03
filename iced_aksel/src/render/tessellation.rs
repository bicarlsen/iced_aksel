use iced::{
    Color,
    advanced::graphics::{color::pack, mesh::SolidVertex2D},
};
use lyon_path::{LineCap, LineJoin, PathEvent, iterator::FromPolyline, traits::PathIterator};
use lyon_tessellation::{
    FillOptions, FillTessellator, FillVertex, FillVertexConstructor, StrokeOptions,
    StrokeTessellator, StrokeVertex, StrokeVertexConstructor,
};

use crate::{
    Stroke,
    render::{MeshBuffer, dashed::DashedPolyline},
    stroke::StrokeStyle,
};

#[derive(Default)]
pub struct Tessellators {
    pub fill: FillTessellator,
    pub stroke: StrokeTessellator,
}

impl Tessellators {
    // 1. The Core Implementation (Handles Dashing logic)
    pub fn stroke_polyline<I, D>(
        &mut self,
        buffer: &mut MeshBuffer,
        points: I,
        stroke: &Stroke<D>,
        resolved_width: f32,
        close_path: bool,
    ) where
        I: IntoIterator<Item = lyon::math::Point>,
    {
        let options = StrokeOptions::default()
            .with_line_width(resolved_width)
            .with_line_cap(LineCap::Butt)
            .with_line_join(LineJoin::Miter);

        let mut writer = buffer.writer(SolidVertexConstructor {
            color: pack(stroke.fill),
        });

        match &stroke.style {
            StrokeStyle::Solid => {
                let _ = self.stroke.tessellate(
                    FromPolyline::new(close_path, points.into_iter()),
                    &options,
                    &mut writer,
                );
            }
            StrokeStyle::Dashed => {
                let dashes = [resolved_width * 5., resolved_width * 2.];
                let dashed = DashedPolyline::new(points.into_iter(), &dashes);
                let _ = self.stroke.tessellate(dashed, &options, &mut writer);
            }
            StrokeStyle::Dotted => {
                let dots = [resolved_width, resolved_width * 2.0];
                let dashed = DashedPolyline::new(points.into_iter(), &dots);
                let _ = self.stroke.tessellate(dashed, &options, &mut writer);
            }
        }
    }

    // 2. The Universal Adapter (Handles Circles/Rects/Paths)
    /// Flattens curves into points, then delegates to stroke_polyline.
    pub fn stroke_path<Iter, D>(
        &mut self,
        buffer: &mut MeshBuffer,
        path: Iter,
        stroke: &Stroke<D>,
        resolved_width: f32,
        tolerance: f32,
    ) where
        Iter: PathIterator,
    {
        // Flatten curves into a list of points
        let points: Vec<lyon::math::Point> = path
            .flattened(tolerance)
            .filter_map(|evt| match evt {
                PathEvent::Begin { at } => Some(at),
                PathEvent::Line { to, .. } => Some(to),
                _ => None,
            })
            .collect();

        // Reuse the polyline logic (which now supports dashing)
        self.stroke_polyline(
            buffer,
            points,
            stroke,
            resolved_width,
            true, // Shapes like circles are usually closed
        );
    }

    /// Flattens a path (like a Circle) and fills it as a solid polygon.
    pub fn fill_path<Iter>(
        &mut self,
        buffer: &mut MeshBuffer,
        path: Iter,
        color: Color,
        tolerance: f32,
    ) where
        Iter: lyon::path::iterator::PathIterator,
    {
        // 1. Flatten curves into points
        let points: Vec<lyon::math::Point> = path
            .flattened(tolerance)
            .filter_map(|evt| match evt {
                lyon::path::PathEvent::Begin { at } => Some(at),
                lyon::path::PathEvent::Line { to, .. } => Some(to),
                _ => None,
            })
            .collect();

        // 2. Delegate to your existing fill_polygon logic
        self.fill_polygon(buffer, points, color);
    }

    // --- FILL API ---

    pub fn fill_polygon<I>(&mut self, buffer: &mut MeshBuffer, points: I, color: Color)
    where
        I: IntoIterator<Item = lyon::math::Point>,
    {
        let options = FillOptions::default();

        let mut writer = buffer.writer(SolidVertexConstructor { color: pack(color) });

        let _ = self.fill.tessellate(
            FromPolyline::new(true, points.into_iter()),
            &options,
            &mut writer,
        );
    }
}

#[derive(Copy, Clone)]
pub struct SolidVertexConstructor {
    pub color: iced::advanced::graphics::color::Packed,
}

impl StrokeVertexConstructor<SolidVertex2D> for SolidVertexConstructor {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> SolidVertex2D {
        SolidVertex2D {
            position: vertex.position().to_array(),
            color: self.color,
        }
    }
}

// ADDED: Logic to handle fills
impl FillVertexConstructor<SolidVertex2D> for SolidVertexConstructor {
    fn new_vertex(&mut self, vertex: FillVertex) -> SolidVertex2D {
        SolidVertex2D {
            position: vertex.position().to_array(),
            color: self.color,
        }
    }
}
