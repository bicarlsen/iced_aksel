use aksel::{Float, PlotPoint};
use iced::{
    Color,
    advanced::Text,
    widget::text::{LineHeight, Shaping, Wrapping},
};

use crate::{Shape, plot};

#[derive(Debug, Clone)]
pub struct Label<D> {
    pub content: String,
    pub position: PlotPoint<D>,
    pub horizontal_alignment: iced::alignment::Horizontal,
    pub vertical_alignment: iced::alignment::Vertical,
    pub fill: Color,
    pub font_size: f32,
}

impl<D: Float, R: plot::Renderer> Shape<D, R> for Label<D> {
    fn render(self, ctx: &mut plot::Context<'_, D, R>) {
        ctx.render_text(move |transform, renderer| {
            // 1. Convert the data-space PlotPoint to a screen-space Point.
            let screen_point = transform.chart_to_screen(&self.position.clone());
            let position = iced::Point {
                x: screen_point.x,
                y: screen_point.y,
            };

            // 2. Create the advanced Text primitive.
            // TODO: Clone text every time? m
            let text = Text {
                content: self.content.clone(),
                bounds: iced::Size::new(500., 500.), // TODO: Better way to handle this. INFINITE doesnt work properly
                size: self.font_size.into(),
                line_height: LineHeight::default(),
                font: iced::Font::default(),
                align_x: self.horizontal_alignment.into(),
                align_y: self.vertical_alignment,
                shaping: Shaping::Basic,
                wrapping: Wrapping::None,
            };

            let screen_bounds = transform.screen_bounds();
            let clip_bounds = iced::Rectangle {
                x: screen_bounds.x,
                y: screen_bounds.y,
                width: screen_bounds.width,
                height: screen_bounds.height,
            };

            // 3. Fill the text on the screen.
            renderer.fill_text(text, position, self.fill, clip_bounds);
        });
    }
}

impl<D: Float> Label<D> {
    pub const fn simple(content: String, position: PlotPoint<D>, fill: Color) -> Self {
        Self {
            content,
            position,
            horizontal_alignment: iced::alignment::Horizontal::Left,
            vertical_alignment: iced::alignment::Vertical::Center,
            fill,
            font_size: 16.,
        }
    }
}
