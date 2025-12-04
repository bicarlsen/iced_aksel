use std::{collections::HashSet, hash::Hash};

use aksel::{Float, PlotPoint, PlotRect};
use derivative::Derivative;
use indexmap::IndexMap;

use crate::Axis;

// TODO: Check out if we need this again. Removed because of compilation error i dont understand
#[derive(Default, Derivative)]
#[derivative(Debug)]
pub struct State<AxisId: Hash + Eq, Domain> {
    axes: IndexMap<AxisId, Axis<Domain>>,
    // potentially more settings?
    // and/or a history of what axis/series-id's we had last frame?
}

impl<AxisId, Domain> State<AxisId, Domain>
where
    AxisId: Hash + Eq + Clone,
    Domain: Float,
{
    pub fn new() -> Self {
        Self {
            axes: IndexMap::new(),
        }
    }

    pub fn get_axis(&self, id: &AxisId) -> Option<&Axis<Domain>> {
        self.axes.get(id)
    }

    pub fn get_axis_mut(&mut self, id: &AxisId) -> Option<&mut Axis<Domain>> {
        self.axes.get_mut(id)
    }

    pub fn set_axis(&mut self, id: impl Into<AxisId>, axis: Axis<Domain>) {
        self.axes.insert(id.into(), axis);
    }

    pub const fn axes(&self) -> &IndexMap<AxisId, Axis<Domain>> {
        &self.axes
    }

    pub const fn axes_mut(&mut self) -> &mut IndexMap<AxisId, Axis<Domain>> {
        &mut self.axes
    }

    pub fn retain_axes(&mut self, active_axes: &[AxisId]) {
        self.axes.retain(|k, _| active_axes.contains(k));
    }

    pub fn visible_axes(&self) -> impl Iterator<Item = (&AxisId, &Axis<Domain>)> {
        self.axes.iter().filter(|(_, axis)| axis.is_visible())
    }

    pub fn pan_scales(&mut self, x_scale: AxisId, y_scale: AxisId, dx: Domain, dy: Domain) {
        if let Some(axis) = self.axes.get_mut(&x_scale) {
            axis.scale_mut().pan(dx);
        }
        if let Some(axis) = self.axes.get_mut(&y_scale) {
            axis.scale_mut().pan(dy);
        }
    }

    pub fn zoom_scales(
        &mut self,
        x_scale: AxisId,
        y_scale: AxisId,
        x_norm: Domain,
        y_norm: Domain,
        factor: Domain,
    ) {
        if let Some(axis) = self.axes.get_mut(&x_scale) {
            axis.scale_mut().zoom(factor, Some(x_norm));
        }
        if let Some(axis) = self.axes.get_mut(&y_scale) {
            axis.scale_mut().zoom(factor, Some(y_norm));
        }
    }

    #[deprecated = "Use State::axes() instead"]
    pub const fn axis(&self) -> &IndexMap<AxisId, Axis<Domain>> {
        &self.axes
    }

    pub fn get_scales_plotrectangle(
        &self,
        x_scale: AxisId,
        y_scale: AxisId,
    ) -> Option<PlotRect<Domain>> {
        let horizontal_range = self.axes.get(&x_scale)?.domain();
        let vertical_range = self.axes.get(&y_scale)?.domain();

        let top_left = PlotPoint::new(*horizontal_range.0, *vertical_range.0);
        let bot_right = PlotPoint::new(*horizontal_range.1, *vertical_range.1);

        Some(PlotRect::from_points(top_left, bot_right))
    }
}
