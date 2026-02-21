use crate::interaction::InteractionId;
use crate::{
    Measure, Shape, Stroke,
    interaction::{HitGeometry, Interaction, InteractiveHitbox, Propagation},
    plot,
    render::Primitive,
};
use aksel::{Float, PlotPoint, PlotRect};
use iced_core::{Color, Point};

#[derive(Debug, Clone)]
enum Geometry<D> {
    Corners {
        p1: PlotPoint<D>,
        p2: PlotPoint<D>,
    },
    Centered {
        center: PlotPoint<D>,
        width: Measure<D>,
        height: Measure<D>,
    },
}

#[derive(Debug, Clone)]
pub struct Rectangle<D, Message = ()> {
    geometry: Geometry<D>,
    pub fill: Option<Color>,
    pub stroke: Option<Stroke<D>>,

    // NEW: Interaction fields!
    pub id: Option<InteractionId>,
    pub on_hover: Option<Message>,
    pub on_click: Option<Message>,
    pub propagation: Propagation,
}

impl<D: Float, Message: Clone, R: crate::Renderer> Shape<D, Message, R> for Rectangle<D, Message> {
    fn render(self, ctx: &mut plot::Context<'_, D, Message, R>) {
        let Self {
            geometry,
            fill,
            stroke,
            id,
            on_hover,
            on_click,
            propagation,
        } = self;

        // 1. Calculate visual screen coordinates
        let (screen_min, screen_max) = match &geometry {
            Geometry::Corners { p1, p2 } => {
                let x1 = ctx.x_to_screen(&p1.x);
                let y1 = ctx.y_to_screen(&p1.y);
                let x2 = ctx.x_to_screen(&p2.x);
                let y2 = ctx.y_to_screen(&p2.y);

                (
                    Point::new(x1.min(x2), y1.min(y2)),
                    Point::new(x1.max(x2), y1.max(y2)),
                )
            }
            Geometry::Centered {
                center,
                width,
                height,
            } => {
                let center_x = ctx.x_to_screen(&center.x);
                let center_y = ctx.y_to_screen(&center.y);

                let width_pixels = width.resolve_x(ctx);
                let height_pixels = height.resolve_y(ctx);

                let half_width = width_pixels / 2.0;
                let half_height = height_pixels / 2.0;

                (
                    Point::new(center_x - half_width, center_y - half_height),
                    Point::new(center_x + half_width, center_y + half_height),
                )
            }
        };

        // 2. Register Interactions with the Broad Phase Registry!
        if on_hover.is_some() || on_click.is_some() {
            // Simply construct the screen rectangle
            let aabb = iced_core::Rectangle {
                x: screen_min.x,
                y: screen_min.y,
                width: screen_max.x - screen_min.x,
                height: screen_max.y - screen_min.y,
            };

            ctx.interactions.add(InteractiveHitbox {
                id,
                aabb,
                // For the MVP, we can just pass a dummy Rect to the geometry
                // since we're only relying on the AABB for a simple rectangle.
                // TODO: Stop relying on this dummy Rect
                geometry: HitGeometry::Rect(PlotRect {
                    x: D::from(0).unwrap(),
                    y: D::from(0).unwrap(),
                    width: D::from(0).unwrap(),
                    height: D::from(0).unwrap(),
                }),
                on_hover: on_hover.map(|msg| Interaction {
                    message: msg,
                    propagation,
                }),
                on_click: on_click.map(|msg| Interaction {
                    message: msg,
                    propagation,
                }),
            });
        }

        // 3. Dispatch visual rendering
        let stroke = stroke.map(|s| s.resolve(ctx));

        ctx.add_primitive(Primitive::Rectangle {
            xy1: screen_min,
            xy2: screen_max,
            fill,
            stroke,
        });
    }
}

impl<D: Float, Message> Rectangle<D, Message> {
    pub const fn corners(p1: PlotPoint<D>, p2: PlotPoint<D>) -> Self {
        Self {
            geometry: Geometry::Corners { p1, p2 },
            fill: None,
            stroke: None,
            id: None,
            on_hover: None,
            on_click: None,
            propagation: Propagation::Stop,
        }
    }

    pub const fn centered(center: PlotPoint<D>, width: Measure<D>, height: Measure<D>) -> Self {
        Self {
            geometry: Geometry::Centered {
                center,
                width,
                height,
            },
            fill: None,
            stroke: None,
            id: None,
            on_hover: None,
            on_click: None,
            propagation: Propagation::Stop,
        }
    }

    pub const fn fill(mut self, color: Color) -> Self {
        self.fill = Some(color);
        self
    }

    pub const fn stroke(mut self, stroke: Stroke<D>) -> Self {
        self.stroke = Some(stroke);
        self
    }

    // --- NEW BUILDERS ---

    // --- NEW BUILDER METHOD ---
    pub fn id(mut self, id: impl std::hash::Hash) -> Self {
        self.id = Some(InteractionId::new(id));
        self
    }

    pub fn on_click(mut self, message: Message) -> Self {
        self.on_click = Some(message);
        self
    }

    pub fn on_hover(mut self, message: Message) -> Self {
        self.on_hover = Some(message);
        self
    }

    pub fn propagation(mut self, propagation: Propagation) -> Self {
        self.propagation = propagation;
        self
    }
}
