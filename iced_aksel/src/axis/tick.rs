use aksel::Tick;
use iced::{Pixels, Point, advanced::text::paragraph::Plain};

use crate::axis::tick::label::{Label, LabelBounds};

pub mod label;

/// Defines the visual styling and content of a single tick mark on an Axis.
///
/// # The Tick Rendering Pipeline
///
/// Understanding `TickLine` requires understanding how `aksel` processes axes.
/// The relationship flows as follows:
///
/// 1.  **Generation (`Scale`):** The [`Scale`] (e.g., `Linear`, `Log`) calculates logical positions
///     and importance levels for potential ticks, creating [`Tick`] objects.
/// 2.  **Contextualization (`Axis`):** The Axis wraps each logical [`Tick`] into a [`TickLabelContext`].
///     This provides context like the axis bounds, domain, and orientation.
/// 3.  **Styling (User Logic):** The user provides a closure via [`Axis::with_tick_renderer`].
///     This closure receives the context and decides **if** and **how** that tick should be drawn
///     by returning an `Option<TickLine>`.
///
/// This separation allows you to have completely dynamic styling—for example, making every
/// major tick thick and red, while making minor ticks thin and grey, or hiding specific ticks entirely.
///
/// [Image of chart axis anatomy showing major and minor ticks]
///
/// # Example
///
/// This example demonstrates how to configure an axis to render major ticks with labels
/// and thick lines, while rendering minor ticks as smaller lines without labels.
///
/// ```rust
/// use iced_aksel::{Axis, axis::TickLine, scale::Linear};
/// use iced::Pixels;
///
/// let axis = Axis::new(Linear::new(0.0, 100.0), iced_aksel::axis::Position::Bottom)
///     .with_tick_renderer(|ctx| {
///         match ctx.tick.level {
///             // Major Tick (Level 0): Thicker, longer, and has a text label
///             0 => Some(TickLine {
///                 thickness: Pixels(2.0),
///                 length: Pixels(10.0),
///                 // We use the context's value to format the text
///                 label: Some(iced_aksel::axis::Label {
///                     content: format!("{:.1}", ctx.tick.value),
///                     ..Default::default()
///                 }),
///             }),
///             // Minor Tick (Level 1): Thinner, shorter, no label
///             1 => Some(TickLine {
///                 thickness: Pixels(1.0),
///                 length: Pixels(5.0),
///                 label: None,
///             }),
///             // Any other importance level: Do not draw (return None)
///             _ => None,
///         }
///     });
/// ```
#[derive(Debug, Clone)]
pub struct TickLine {
    /// The visual thickness (stroke width) of the tick line.
    pub thickness: Pixels,

    /// The length of the tick line perpendicular to the axis.
    pub length: Pixels,

    /// The optional text label associated with this tick.
    ///
    /// If `None`, the tick will be drawn as a simple line with no text.
    /// If provided, the text is usually rendered at the end of the tick line.
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

/// Decision on whether to render or skip a tick label.
///
/// Used by custom label policies provided to [`Axis::with_custom_label_policy`](crate::Axis::with_custom_label_policy).
///
/// # Example
///
/// ```rust
/// use iced_aksel::{Axis, axis::LabelDecision, scale::Linear};
///
/// // Only show labels for even values
/// let axis = Axis::new(Linear::new(0.0, 100.0), iced_aksel::axis::Position::Bottom)
///     .with_custom_label_policy(|ctx| {
///         if ctx.tick.value as i32 % 2 == 0 {
///             LabelDecision::Render
///         } else {
///             LabelDecision::Skip
///         }
///     });
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelDecision {
    /// Render this label at its position.
    Render,
    /// Skip rendering this label (e.g., due to overlap or custom filtering).
    Skip,
}

pub struct LabelCandidate<D> {
    pub(crate) tick: Tick<D>,
    pub(crate) normalized_position: f32,
    pub(crate) label: Label,
}

pub struct ResolvedLabelCandidate<Renderer, D>
where
    Renderer: iced::advanced::text::Renderer,
{
    pub(crate) tick: Tick<D>,
    pub(crate) normalized_position: f32,
    pub(crate) bounds: LabelBounds,
    pub(crate) paragraph: Plain<Renderer::Paragraph>,
    pub(crate) position: Point,
}
