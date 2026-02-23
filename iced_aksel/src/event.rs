use iced_core::{keyboard, mouse};

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
