use aksel::{Float, Transform};
use derivative::Derivative;
use iced_core::{Point, Rectangle, keyboard};
use indexmap::IndexMap;
use rapidhash::fast::RandomState;

use crate::event::{self, PressEvent, ReleaseEvent};

mod area;
mod id;

pub use area::Area;
pub use id::Id;

use area::ResolvedArea;

type HoverHandler<Message> = event::Handler<Message, (Id, keyboard::Modifiers)>;
type DragHandler<Message> = event::Handler<Message, (Id, event::DragEvent<event::Delta>)>;
type PressHandler<Message> = event::Handler<Message, (Id, PressEvent<Point>)>;
type ReleaseHandler<Message> = event::Handler<Message, (Id, ReleaseEvent<Point>)>;

pub struct Interaction<D, Message: Clone> {
    pub(crate) id: Id,
    pub(crate) area: Area<D>,
    pub(crate) on_hover: Option<HoverHandler<Message>>,
    pub(crate) on_drag: Option<DragHandler<Message>>,
    pub(crate) on_press: Option<PressHandler<Message>>,
    pub(crate) on_release: Option<ReleaseHandler<Message>>,
}

impl<D: Float, Message: Clone> Interaction<D, Message> {
    pub(crate) fn resolve(
        self,
        transform: &Transform<D, f32, f32>,
    ) -> (Id, ResolvedInteraction<Message>) {
        let Self {
            id,
            area,
            on_hover,
            on_drag,
            on_press,
            on_release,
        } = self;

        let area = area.resolve(transform);
        let bounding_box = area.bounding_box();

        (
            id,
            ResolvedInteraction {
                area,
                bounding_box,
                on_hover,
                on_drag,
                on_press,
                on_release,
            },
        )
    }

    pub fn new(id: impl Into<Id>, area: impl Into<Area<D>>) -> Self {
        let id = id.into();
        let area = area.into();
        Self {
            id,
            area,
            on_hover: None,
            on_drag: None,
            on_press: None,
            on_release: None,
        }
    }

    event::impl_handlers!(
        /// Sets the event handler for interaction hovering
        hover: (Id, keyboard::Modifiers);

        /// Sets the event handler for interaction dragging
        drag: (Id, event::DragEvent<event::Delta>);

        /// Sets the event handler for interaction mouse presses
        press: (Id, PressEvent<Point>);

        /// Sets the event handler for interaction mouse releases
        release: (Id, ReleaseEvent<Point>);
    );
}

/// A stored interaction waiting to be tested against mouse events.
#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct ResolvedInteraction<Message: Clone> {
    pub area: ResolvedArea,
    pub bounding_box: Rectangle,

    #[derivative(Debug = "ignore")]
    pub on_hover: Option<HoverHandler<Message>>,
    #[derivative(Debug = "ignore")]
    pub on_drag: Option<DragHandler<Message>>,
    #[derivative(Debug = "ignore")]
    pub on_press: Option<PressHandler<Message>>,
    #[derivative(Debug = "ignore")]
    pub on_release: Option<ReleaseHandler<Message>>,
}

/// The registry that collects hitboxes during the drawing phase.
#[derive(Debug)]
pub(crate) struct InteractionsCache<Message: Clone>(
    IndexMap<Id, ResolvedInteraction<Message>, RandomState>,
);

impl<Message: Clone> InteractionsCache<Message> {
    pub fn new() -> Self {
        Self(IndexMap::with_hasher(RandomState::new()))
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, Id, ResolvedInteraction<Message>> {
        self.0.iter()
    }

    pub fn get(&self, id: &Id) -> Option<&ResolvedInteraction<Message>> {
        self.0.get(id)
    }

    /// Push an interaction to the cache
    pub fn insert(&mut self, id: Id, interaction: ResolvedInteraction<Message>) {
        self.0.insert(id, interaction);
    }

    // Clear the inner vector, re-using the allocated space next time we push
    pub fn clear(&mut self) {
        self.0.clear();
    }
}
