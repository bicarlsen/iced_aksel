use std::cell::{RefCell, RefMut};

use super::Action;
use crate::{
    Quality,
    render::{RenderBuffer, Renderer},
};

use iced_core::mouse;

/// Internal chart memory
pub struct Memory<AxisId> {
    pub action: Action<AxisId>,
    pub previous_click: Option<mouse::Click>,
    pub buffer: Option<RefCell<RenderBuffer>>,
}

impl<AxisId> Memory<AxisId> {
    pub fn new() -> Self {
        Self {
            action: Action::default(),
            previous_click: None,
            buffer: None,
        }
    }

    pub fn make_sure_buffer_is_initialized<R: Renderer>(&mut self, renderer: &R, quality: Quality) {
        if let Some(buffer) = &self.buffer {
            buffer.borrow_mut().set_quality(quality);
        } else {
            let mut buffer = renderer.init_buffer();
            buffer.set_quality(quality);
            self.buffer = Some(RefCell::new(buffer));
        }
    }

    /// Gets the internal buffer
    ///
    /// Panics if the buffer isn't initialized
    pub fn get_buffer(&self) -> RefMut<RenderBuffer> {
        self.buffer
            .as_ref()
            .expect("Buffer isn't initialized")
            .borrow_mut()
    }
}
