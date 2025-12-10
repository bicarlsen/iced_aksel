use std::{collections::HashSet, hash::Hash};

use aksel::{Float, PlotPoint, PlotRect};
use derivative::Derivative;
use indexmap::{
    IndexMap,
    map::{Iter, IterMut},
};

use crate::Axis;

#[derive(Default, Derivative)]
#[derivative(Debug)]
/// TODO: Document. Make big explanatory comments
pub struct State<AxisId: Hash + Eq, Domain> {
    axes: IndexMap<AxisId, Axis<Domain>>,
}

impl<AxisId, D> State<AxisId, D>
where
    AxisId: Hash + Eq + Clone,
    D: Float,
{
    pub fn new() -> Self {
        Self {
            axes: IndexMap::new(),
        }
    }

    pub fn get_axes(&self) -> &IndexMap<AxisId, Axis<D>> {
        &self.axes
    }

    /// Builder style: `State::new().with_axis(...)`
    pub fn with_axis(mut self, id: impl Into<AxisId>, axis: Axis<D>) -> Self {
        self.axes.insert(id.into(), axis);
        self
    }

    /// Runtime addition: `state.set_axis
    /// (...)`
    pub fn set_axis(&mut self, id: impl Into<AxisId>, axis: Axis<D>) -> Option<Axis<D>> {
        self.axes.insert(id.into(), axis)
    }

    pub fn remove_axis(&mut self, id: &AxisId) -> Option<Axis<D>> {
        self.axes.remove(id)
    }

    pub fn has_axis(&self, id: &AxisId) -> bool {
        self.axes.contains_key(id)
    }

    // -------------------------------------------------------------------------
    // B. Data Access
    // -------------------------------------------------------------------------

    /// Get a read-only reference to an axis.
    pub fn axis(&self, id: &AxisId) -> Option<&Axis<D>> {
        self.axes.get(id)
    }

    /// Get a mutable reference to an axis.
    pub fn axis_mut(&mut self, id: &AxisId) -> Option<&mut Axis<D>> {
        self.axes.get_mut(id)
    }

    /// Iterates over all axes mutably.
    /// If you need to modify multiple axes (e.g. X and Y) safely, use this iterator.
    pub fn axes_iter_mut(&mut self) -> IterMut<'_, AxisId, Axis<D>> {
        self.axes.iter_mut()
    }

    // -------------------------------------------------------------------------
    // C. Coordinated Logic Helpers (The "Controller")
    // -------------------------------------------------------------------------

    /// Helper: Get the current domain of a specific axis.
    pub fn domain(&self, id: &AxisId) -> Option<(&D, &D)> {
        self.axes.get(id).map(|a| a.scale().domain())
    }

    /// Helper: Manually set the domain of an axis.
    pub fn set_domain(&mut self, id: &AxisId, min: D, max: D) {
        if let Some(axis) = self.axes.get_mut(id) {
            axis.scale_mut().set_domain(min, max);
        }
    }

    // /// Helper: Pan X and Y axes simultaneously using normalized deltas.
    // pub fn pan_axes(
    //     &mut self,
    //     x_id: &AxisId,
    //     y_id: &AxisId,
    //     delta_x: Normalized<D>,
    //     delta_y: Normalized<D>,
    // ) {
    //     for (id, axis) in self.axes.iter_mut() {
    //         if id == x_id {
    //             axis.scale_mut().pan(delta_x);
    //         } else if id == y_id {
    //             // simple 'else if' allows generic handling even if x_id == y_id
    //             // (though usually they are different)
    //             axis.scale_mut().pan(delta_y);
    //         }
    //     }
    // }

    // /// Helper: Zoom X and Y axes around a normalized anchor point.
    // pub fn zoom_axes(
    //     &mut self,
    //     x_id: &AxisId,
    //     y_id: &AxisId,
    //     factor: D,
    //     anchor_x: Option<Normalized<D>>,
    //     anchor_y: Option<Normalized<D>>,
    // ) {
    //     // Validation
    //     let norm_factor = if let Some(nf) = Normalized::new(factor) {
    //         nf
    //     } else {
    //         return;
    //     };

    //     for (id, axis) in self.axes.iter_mut() {
    //         if id == x_id {
    //             axis.scale_mut().zoom(norm_factor, anchor_x);
    //         } else if id == y_id {
    //             axis.scale_mut().zoom(norm_factor, anchor_y);
    //         }
    //     }
    // }
}
