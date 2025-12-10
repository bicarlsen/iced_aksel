use aksel::{Float, Tick};
use iced::{Pixels, Point, Rectangle, advanced::text::paragraph::Plain};

use crate::axis::tick::label::{Label, LabelBounds};

pub mod label;

#[derive(Debug, Clone)]
pub struct TickLine {
    pub thickness: Pixels,
    pub length: Pixels,
    pub label: Option<Label>,
}

impl Default for TickLine {
    #[inline(always)]
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
            length: Pixels(5.0),
            label: None,
        }
    }
}

impl TickLine {
    #[inline(always)]
    pub fn simple(content: String) -> Self {
        Self {
            label: Some(Label {
                content,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlacedLabelInfo<D> {
    pub tick: Tick<D>,
    pub normalized_position: f32,
    pub bounds: LabelBounds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecision {
    Render,
    Skip,
}

pub(crate) struct LabelCandidate<D> {
    pub(crate) tick: Tick<D>,
    pub(crate) normalized_position: f32,
    pub(crate) label: Label,
}

pub(crate) struct ResolvedLabelCandidate<Renderer, D>
where
    Renderer: iced::advanced::text::Renderer,
{
    pub(crate) tick: Tick<D>,
    pub(crate) normalized_position: f32,
    pub(crate) bounds: LabelBounds,
    pub(crate) paragraph: Plain<Renderer::Paragraph>,
    pub(crate) position: Point,
}
