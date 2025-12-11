use crate::Measure;

use iced::Color;

/// Represents the style of a stroke. These are all predifined for now, and has no further customizability.
///
/// Strokes width will affect the look of each style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeStyle {
    /// Will render stroke as solid lines, no gaps
    Solid,
    /// Will render stroke as dashed lines
    Dashed,
    /// Will render stroke as dotted lines
    Dotted,
}

/// Represents a stroke with a fill color, thickness, and style.
#[derive(Debug, Clone, Copy)]
pub struct Stroke<D> {
    pub fill: Color,
    pub thickness: Measure<D>,
    pub style: StrokeStyle,
}

impl<D> Stroke<D> {
    /// Creates a new stroke with the given fill color and thickness. Defaults style to `StrokeStyle::Solid`
    pub const fn new(fill: Color, thickness: Measure<D>) -> Self {
        Self {
            fill,
            thickness,
            style: StrokeStyle::Solid,
        }
    }

    /// Creates a new stroke with the given fill color, thickness, and style.
    pub const fn with_style(fill: Color, thickness: Measure<D>, style: StrokeStyle) -> Self {
        Self {
            fill,
            thickness,
            style,
        }
    }
}
