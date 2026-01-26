use super::Action;
use iced_core::mouse;

/// Internal chart memory
pub struct Memory<AxisId> {
    pub action: Action<AxisId>,
    pub previous_click: Option<mouse::Click>,
}

impl<AxisId> Memory<AxisId> {
    pub fn new() -> Self {
        Self {
            action: Action::default(),
            previous_click: None,
        }
    }
}
