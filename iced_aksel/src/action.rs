use crate::interaction;
use iced_core::Point;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Action<AxisId> {
    #[default]
    Idle,
    DraggingPlot {
        interaction_id: Option<interaction::Id>,
        origin: Point,
        last_position: Point,
        total_delta: f32,
    },
    DraggingAxis {
        id: AxisId,
        origin: f32,
        last_position: f32,
        total_delta: f32,
    },
}

impl<AxisId> Action<AxisId> {
    pub(crate) const fn total_drag_delta(&self) -> Option<f32> {
        match self {
            Self::Idle => None,
            Self::DraggingPlot { total_delta, .. } => Some(*total_delta),
            Self::DraggingAxis { total_delta, .. } => Some(*total_delta),
        }
    }
}
