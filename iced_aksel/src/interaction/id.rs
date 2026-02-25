use std::{
    borrow,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

/// An identifier for an interaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(Internal);

impl Id {
    pub const fn new(id: &'static str) -> Self {
        Self(Internal::Custom(borrow::Cow::Borrowed(id)))
    }

    pub fn unique() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        Self(Internal::Unique(id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Internal {
    Unique(usize),
    Custom(borrow::Cow<'static, str>),
}
