// src/interaction.rs
use aksel::{Float, PlotPoint, PlotRect};
use iced_core::Rectangle;

/// Determines if an event should stop propagating or pass through to shapes behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Propagation {
    #[default]
    Stop,
    PassThrough,
}

/// The exact geometric intent for the hit-test.
#[derive(Debug, Clone)]
pub enum HitGeometry<D> {
    /// A simple data-space bounding box (e.g., filled Rectangle)
    Rect(PlotRect<D>),
    /// A line segment with a pixel-based thickness for the stroke
    LineSegment {
        p1: PlotPoint<D>,
        p2: PlotPoint<D>,
        stroke_width_px: f32,
    },
}

/// An interaction configuration attached to a shape.
#[derive(Debug, Clone)]
pub struct Interaction<Message> {
    pub message: Message,
    pub propagation: Propagation,
}

/// A stored hitbox waiting to be tested against mouse events.
#[derive(Debug)]
pub struct InteractiveHitbox<D, Message> {
    /// The broad-phase bounding box
    /// TODO: Consider using a plot-coordinate aware struct?
    pub aabb: Rectangle,
    /// The narrow-phase precise geometry
    pub geometry: HitGeometry<D>,

    pub on_hover: Option<Interaction<Message>>,
    pub on_click: Option<Interaction<Message>>,
}

/// The registry that collects hitboxes during the drawing phase.
#[derive(Debug)]
pub struct InteractionRegistry<D, Message> {
    pub(crate) hitboxes: Vec<InteractiveHitbox<D, Message>>,
}

impl<D, Message> InteractionRegistry<D, Message> {
    pub fn new() -> Self {
        Self {
            hitboxes: Vec::new(),
        }
    }

    pub fn add(&mut self, hitbox: InteractiveHitbox<D, Message>) {
        self.hitboxes.push(hitbox);
    }

    pub fn clear(&mut self) {
        self.hitboxes.clear();
    }
}

impl<D, Message> Default for InteractionRegistry<D, Message> {
    fn default() -> Self {
        Self::new()
    }
}
