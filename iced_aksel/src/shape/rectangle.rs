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
            // To ensure zooming doesn't break hitboxes defined in Screen pixels,
            // we reverse-project the calculated screen bounds back to Data space!
            let dx1 = ctx.x_from_screen(&screen_min.x);
            let dx2 = ctx.x_from_screen(&screen_max.x);
            let dy1 = ctx.x_from_screen(&screen_min.y);
            let dy2 = ctx.x_from_screen(&screen_max.y);

            let min_x = if dx1 < dx2 { dx1 } else { dx2 };
            let max_x = if dx1 > dx2 { dx1 } else { dx2 };
            let min_y = if dy1 < dy2 { dy1 } else { dy2 };
            let max_y = if dy1 > dy2 { dy1 } else { dy2 };

            let data_bounds = PlotRect {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            };

            ctx.interactions.add(InteractiveHitbox {
                aabb: data_bounds.clone(),
                geometry: HitGeometry::Rect(data_bounds),
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
