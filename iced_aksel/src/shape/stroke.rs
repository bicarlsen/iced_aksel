use crate::Length;

use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrokeStyle {
    Solid,
    /// Custom pattern: [on_pixels, off_pixels, on_pixels...]
    Dashed,
    /// Standard dotted: 1x width dot, 2x width gap
    Dotted,
}

#[derive(Debug, Clone, Copy)]
pub struct Stroke<D> {
    pub fill: Color,
    pub thickness: Length<D>,
    pub style: StrokeStyle,
}

impl<D> Stroke<D> {
    pub const fn new(fill: Color, thickness: Length<D>) -> Self {
        Self {
            fill,
            thickness,
            style: StrokeStyle::Solid,
        }
    }

    pub const fn dashed(mut self) -> Self {
        self.style = StrokeStyle::Dashed;
        self
    }

    pub const fn dotted(mut self) -> Self {
        self.style = StrokeStyle::Dotted;
        self
    }
}
