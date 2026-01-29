use std::cell::{RefCell, RefMut};

use super::Action;
use crate::{
    Quality,
    render::{Backend, RenderBuffer},
};

use iced_core::mouse;

/// Internal chart memory
pub struct Memory<AxisId, Renderer: crate::Renderer> {
    pub action: Action<AxisId>,
    pub previous_click: Option<mouse::Click>,
    pub buffer: Option<RefCell<RenderBuffer<Renderer>>>,
}

impl<AxisId, Renderer: crate::Renderer> Memory<AxisId, Renderer> {
    pub fn new() -> Self {
        Self {
            action: Action::default(),
            previous_click: None,
            buffer: None,
        }
    }

    pub fn make_sure_buffer_is_initialized(&mut self, renderer: &Renderer, quality: Quality) {
        if let Some(buffer) = &self.buffer {
            buffer.borrow_mut().set_quality(quality);
        } else {
            let mut buffer = match renderer.preffered_backend() {
                Backend::Mesh => RenderBuffer::new_mesh(100_000),
                Backend::Path => RenderBuffer::new_path(5000),
            };
            buffer.set_quality(quality);
            self.buffer = Some(RefCell::new(buffer));
        }
    }

    /// Gets the internal buffer
    ///
    /// Panics if the buffer isn't initialized
    pub fn get_buffer(&self) -> RefMut<'_, RenderBuffer<Renderer>> {
        self.buffer
            .as_ref()
            .expect("Buffer isn't initialized")
            .borrow_mut()
    }
}
