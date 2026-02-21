use std::hash::{Hash, Hasher};
// src/interaction.rs
use aksel::{Float, PlotPoint, PlotRect};
use iced_core::Rectangle;

/// A unique identifier for an interactive shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractionId(pub u64);

impl InteractionId {
    pub fn new<T: Hash>(id: T) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        id.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// Identifies what is currently being hovered, preferring explicit IDs over array indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoverIdentity {
    Id(InteractionId),
    Index(usize),
}

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
    pub id: Option<InteractionId>,
    /// The broad-phase bounding box
    /// TODO: Consider using a plot-coordinate aware struct?
    pub aabb: Rectangle,
    /// The narrow-phase precise geometry
    pub geometry: HitGeometry<D>,

    pub on_hover: Option<Interaction<Message>>,
    pub on_click: Option<Interaction<Message>>,
    pub on_double_click: Option<Interaction<Message>>,
    pub on_press: Option<Interaction<Message>>,
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
