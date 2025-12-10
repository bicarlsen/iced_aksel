use std::hash::Hash;

use aksel::Float;
use derivative::Derivative;

use super::plot;

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Layer<'a, AxisId, Domain, Renderer, Theme> {
    pub(crate) horizontal_axis_id: AxisId,
    pub(crate) vertical_axis_id: AxisId,

    #[derivative(Debug = "ignore")]
    pub(crate) items: &'a dyn plot::Items<Domain, Renderer, Theme>,
}

impl<'a, AxisId: Hash + Eq, D: Float, R: plot::Renderer, Theme> Layer<'a, AxisId, D, R, Theme> {
    pub const fn new<T: plot::Items<D, R, Theme>>(
        items: &'a T,
        horizontal_axis_id: AxisId,
        vertical_axis_id: AxisId,
    ) -> Self {
        Self {
            horizontal_axis_id,
            vertical_axis_id,
            items,
        }
    }
}
