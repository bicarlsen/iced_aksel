use iced_core::{keyboard, mouse};

#[derive(Debug)]
pub(crate) enum Handler<Message: Clone, F> {
    Direct(Message),
    Closure(F),
}

impl<Message: Clone, F> Handler<Message, F> {
    pub fn run(&self, f: impl Fn(&F) -> Message) -> Message {
        match self {
            Self::Direct(msg) => msg.clone(),
            Self::Closure(closure) => f(closure),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PressEvent<P> {
    pub position: P,
    pub button: mouse::Button,
    pub click_kind: mouse::click::Kind,
    pub modifiers: keyboard::Modifiers,
}

impl<P> PressEvent<P> {
    pub(crate) fn new(
        position: P,
        button: mouse::Button,
        click_kind: mouse::click::Kind,
        modifiers: keyboard::Modifiers,
    ) -> Self {
        Self {
            position,
            button,
            click_kind,
            modifiers,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReleaseEvent<P> {
    pub position: P,
    pub button: mouse::Button,
    pub click_kind: Option<mouse::click::Kind>,
    pub modifiers: keyboard::Modifiers,
}

impl<P> ReleaseEvent<P> {
    pub(crate) fn new(
        position: P,
        button: mouse::Button,
        click_kind: Option<mouse::click::Kind>,
        modifiers: keyboard::Modifiers,
    ) -> Self {
        Self {
            position,
            button,
            click_kind,
            modifiers,
        }
    }
}
