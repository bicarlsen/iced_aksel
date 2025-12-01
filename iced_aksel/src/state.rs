use std::hash::Hash;

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

    pub fn visible_axes(&self) -> impl Iterator<Item = (&AxisId, &Axis<Domain>)> {
        self.axes.iter().filter(|(_, axis)| axis.is_visible())
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

    // pub fn get_plotpoint(
    //     &self,
    //     x_scale: AxisId,
    //     y_scale: AxisId,
    //     normalized: Point<f32>,
    // ) -> Option<PlotPoint<Domain>> {
    //     let hori = self.axis.get(&x_scale)?;
    //     let vert = self.axis.get(&y_scale)?;
    //
    //     let x = hori.denormalize(normalized.x).convert_to();
    //     let y = vert.denormalize(normalized.y).convert_to();
    //
    //     Some(PlotPoint::new(x, y))
    // }
}
