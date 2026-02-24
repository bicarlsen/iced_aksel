use aksel::{Float, PlotPoint, Transform};
use iced_core::{Point, Rectangle};

use crate::Measure;

/// The exact geometric intent for the hit-test.
#[derive(Debug, Clone)]
pub enum Area<D> {
    /// A simple data-space bounding box (e.g., filled Rectangle)
    Rect {
        x: D,
        y: D,
        width: Measure<D>,
        height: Measure<D>,
    },
    /// A line segment with a pixel-based thickness for the stroke
    LineSegment {
        p1: PlotPoint<D>,
        p2: PlotPoint<D>,
        width: f32,
    },
}

impl<D: Float> Area<D> {
    pub(super) fn resolve(self, transform: &Transform<D, f32, f32>) -> ResolvedArea {
        match self {
            Self::Rect {
                x,
                y,
                width,
                height,
            } => {
                // For Plot measures, we need both corners to handle axis inversions (e.g., Y-axis flip)
                let width_data = if let Measure::Plot(w) = width {
                    w
                } else {
                    D::zero()
                };
                let height_data = if let Measure::Plot(h) = height {
                    h
                } else {
                    D::zero()
                };

                let p1 = transform.chart_to_screen(&PlotPoint::new(x, y));
                let p2 =
                    transform.chart_to_screen(&PlotPoint::new(x + width_data, y + height_data));

                ResolvedArea::Rect(Rectangle {
                    x: p1.x.min(p2.x),
                    y: p1.y.min(p2.y),
                    width: width.resolve_x(transform),
                    height: height.resolve_y(transform),
                })
            }
            _ => todo!("Resolve other areas"),
        }
    }
}

#[derive(Debug)]
pub enum ResolvedArea {
    Rect(Rectangle),
    LineSegment {
        p1: Point,
        p2: Point,
        stroke_width_px: f32,
    },
}

impl ResolvedArea {
    pub fn contains(&self, point: Point) -> bool {
        match self {
            Self::Rect(rect) => rect.contains(point),
            _ => todo!("Check if geometry contains point"),
        }
    }

    pub fn bounding_box(&self) -> Rectangle {
        match self {
            Self::Rect(rect) => *rect,
            _ => todo!("Create bounding boxes"),
        }
    }
}
