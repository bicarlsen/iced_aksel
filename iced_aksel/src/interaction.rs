use aksel::{Float, Transform};
use iced_core::Rectangle;
use std::hash::{Hash, Hasher};

mod area;

pub use area::Area;
use area::ResolvedArea;

// TODO: Enforce ID's better!
/// A unique identifier for an interactive shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(pub u64);

impl Id {
    pub fn new<T: Hash>(id: T) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        id.hash(&mut hasher);
        Self(hasher.finish())
    }
}

/// Determines if an event should stop propagating or pass through to shapes behind it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Propagation {
    #[default]
    Stop,
    PassThrough,
}

/// An interaction configuration attached to a shape.
#[derive(Debug, Clone)]
pub struct Event<Message> {
    pub message: Message,
    pub propagation: Propagation,
}

impl<Message> Event<Message> {
    pub const fn new(message: Message) -> Self {
        Self {
            message,
            propagation: Propagation::PassThrough,
        }
    }

    pub const fn stop_propagation(mut self) -> Self {
        self.propagation = Propagation::Stop;
        self
    }
}

pub struct Interaction<D, Message> {
    pub id: Id,
    pub area: Area<D>,
    pub on_hover: Option<Event<Message>>,
    pub on_click: Option<Event<Message>>,
    pub on_double_click: Option<Event<Message>>,
    pub on_press: Option<Event<Message>>,
}

impl<D: Float, Message> Interaction<D, Message> {
    pub(crate) fn resolve(
        self,
        transform: &Transform<D, f32, f32>,
    ) -> ResolvedInteraction<Message> {
        let Self {
            id,
            area,
            on_hover,
            on_click,
            on_double_click,
            on_press,
        } = self;

        let area = area.resolve(transform);
        let bounding_box = area.bounding_box();

        ResolvedInteraction {
            id,
            area,
            bounding_box,
            on_hover,
            on_click,
            on_double_click,
            on_press,
        }
    }

    pub fn new(id: impl Hash, area: impl Into<Area<D>>) -> Self {
        let id = Id::new(id);
        let area = area.into();
        Self {
            id,
            area,
            on_hover: None,
            on_click: None,
            on_double_click: None,
            on_press: None,
        }
    }

    pub fn on_hover(mut self, event: Event<Message>) -> Self {
        self.on_hover = Some(event);
        self
    }

    pub fn on_click(mut self, event: Event<Message>) -> Self {
        self.on_click = Some(event);
        self
    }

    pub fn on_double_click(mut self, event: Event<Message>) -> Self {
        self.on_double_click = Some(event);
        self
    }

    pub fn on_press(mut self, event: Event<Message>) -> Self {
        self.on_press = Some(event);
        self
    }
}

/// A stored interaction waiting to be tested against mouse events.
#[derive(Debug)]
pub(crate) struct ResolvedInteraction<Message> {
    pub id: Id,
    pub area: ResolvedArea,
    pub bounding_box: Rectangle,

    pub on_hover: Option<Event<Message>>,
    pub on_click: Option<Event<Message>>,
    pub on_double_click: Option<Event<Message>>,
    pub on_press: Option<Event<Message>>,
}

/// The registry that collects hitboxes during the drawing phase.
#[derive(Debug)]
pub(crate) struct InteractionsCache<Message>(Vec<ResolvedInteraction<Message>>);

impl<Message> InteractionsCache<Message> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn iter(&self) -> std::slice::Iter<'_, ResolvedInteraction<Message>> {
        self.0.iter()
    }

    /// Push an interaction to the cache
    pub fn push(&mut self, interaction: ResolvedInteraction<Message>) {
        self.0.push(interaction);
    }

    // Clear the inner vector, re-using the allocated space next time we push
    pub fn clear(&mut self) {
        self.0.clear();
    }
}
