use std::hash::Hash;

use aksel::Float;
use derivative::Derivative;
use indexmap::{IndexMap, map::IterMut};

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

    /// Get a read-only reference to the axes.
    pub const fn get_axes(&self) -> &IndexMap<AxisId, Axis<D>> {
        &self.axes
    }

    /// Builder style: `State::new().with_axis(...)`
    pub fn with_axis(mut self, id: impl Into<AxisId>, axis: Axis<D>) -> Self {
        self.axes.insert(id.into(), axis);
        self
    }

    /// Takes in a Id and Axis. Replace the axis if it exists.
    pub fn set_axis(&mut self, id: impl Into<AxisId>, axis: Axis<D>) -> Option<Axis<D>> {
        self.axes.insert(id.into(), axis)
    }

    /// Removes an axis from the state.
    pub fn remove_axis(&mut self, id: &AxisId) -> Option<Axis<D>> {
        self.axes.swap_remove(id)
    }

    /// Checks if an axis exists.
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
    /// If you need to modify multiple axes (e.g. X and Y), use this iterator.
    pub fn axes_iter_mut(&mut self) -> IterMut<'_, AxisId, Axis<D>> {
        self.axes.iter_mut()
    }

    // -------------------------------------------------------------------------
    // C. Coordinated Logic Helpers (The "Controller")
    // -------------------------------------------------------------------------

    /// Get the current domain of a specific axis.
    pub fn domain(&self, id: &AxisId) -> Option<(&D, &D)> {
        self.axes.get(id).map(|a| a.domain())
    }

    pub const fn axes_mut(&mut self) -> &mut IndexMap<AxisId, Axis<D>> {
        &mut self.axes
    }

    pub fn retain_axes(&mut self, active_axes: &[AxisId]) {
        self.axes.retain(|k, _| active_axes.contains(k));
    }

    pub fn visible_axes(&self) -> impl Iterator<Item = (&AxisId, &Axis<D>)> {
        self.axes.iter().filter(|(_, axis)| axis.is_visible())
    }

    pub fn pan_axes(&mut self, x_scale: AxisId, y_scale: AxisId, dx: f32, dy: f32) {
        if let Some(axis) = self.axes.get_mut(&x_scale) {
            axis.pan(dx);
        }
        if let Some(axis) = self.axes.get_mut(&y_scale) {
            axis.pan(dy);
        }
    }

    pub fn zoom_axes(
        &mut self,
        x_scale: AxisId,
        y_scale: AxisId,
        x_anchor_norm: f32,
        y_anchor_norm: f32,
        factor: f32,
    ) {
        if let Some(axis) = self.axes.get_mut(&x_scale) {
            axis.zoom(factor, Some(x_anchor_norm));
        }
        if let Some(axis) = self.axes.get_mut(&y_scale) {
            axis.zoom(factor, Some(y_anchor_norm));
        }
    }

    /// Manually set the domain of an axis.
    pub fn set_domain(&mut self, id: &AxisId, min: D, max: D) {
        if let Some(axis) = self.axes.get_mut(id) {
            axis.set_domain(min, max);
        }
    }
}
