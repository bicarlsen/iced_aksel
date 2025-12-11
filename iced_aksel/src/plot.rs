use aksel::{Float, PlotRect, Transform};

use crate::{
    render::{MeshBuffer, Tessellators},
    shape::Shape,
};

use iced::{Color, Point, advanced::Text};

pub trait Renderer:
    iced::advanced::Renderer
    + iced::advanced::graphics::mesh::Renderer
    + iced::advanced::text::Renderer<Font = iced::Font>
{
}

impl<T> Renderer for T where
    T: iced::advanced::Renderer
        + iced::advanced::graphics::mesh::Renderer
        + iced::advanced::text::Renderer<Font = iced::Font>
{
}

pub trait PlotData<D, R = iced::Renderer, Theme = iced::Theme>
where
    D: Float,
    R: Renderer,
{
    fn draw(&self, plot: &mut Plot<D, R>, theme: &Theme);
}

#[derive(Debug, Clone, Copy)]
enum ShapeType {
    Mesh,
    Text,
}

pub struct TextRenderer<'a, Renderer: iced::advanced::text::Renderer<Font = iced::Font>>(
    &'a mut Renderer,
);

impl<Renderer: iced::advanced::text::Renderer<Font = iced::Font>> TextRenderer<'_, Renderer> {
    pub fn fill_text(
        &mut self,
        text: Text,
        position: Point,
        color: Color,
        clip_bounds: iced::Rectangle,
    ) {
        self.0.fill_text(text, position, color, clip_bounds);
    }
}

pub struct Context<'a, D: Float, Renderer: self::Renderer> {
    transform: &'a Transform<'a, D, f32, f32>,
    clip_bounds: &'a iced::Rectangle,
    renderer: &'a mut Renderer,
    tessellators: &'a mut Tessellators,
    mesh_buffer: &'a mut MeshBuffer,
    last_drawn: ShapeType,
}

impl<'a, D: Float, Renderer: self::Renderer> Context<'a, D, Renderer> {
    #[inline(always)]
    fn reset_layer(&mut self) {
        self.renderer.end_layer();
        self.renderer.start_layer(*self.clip_bounds);
    }

    pub fn render_mesh<F>(&mut self, f: F)
    where
        F: FnOnce(&Transform<'a, D, f32, f32>, &mut MeshBuffer, &mut Tessellators),
    {
        if matches!(self.last_drawn, ShapeType::Text) {
            // Since meshes are always drawn under text, we have to start a new layer in order to
            // ensure Z-ordering
            self.last_drawn = ShapeType::Mesh;
            self.reset_layer();
        }

        // Draw mesh
        f(self.transform, self.mesh_buffer, self.tessellators);

        // If buffer exceeds limit, render the mesh
        if self.mesh_buffer.vertices_count() >= self.mesh_buffer.limit() {
            self.mesh_buffer.render(self.renderer, self.clip_bounds);
        }
    }

    pub fn render_text<F>(&mut self, f: F)
    where
        F: FnOnce(&Transform<'a, D, f32, f32>, &mut TextRenderer<'_, Renderer>),
    {
        if matches!(self.last_drawn, ShapeType::Mesh) {
            // Since text is always drawn over meshes, we don't **have** to start a new layer.
            self.last_drawn = ShapeType::Text;
        }

        let mut renderer = TextRenderer(self.renderer);

        f(self.transform, &mut renderer)
    }
}

pub struct Plot<'a, D: Float, R: self::Renderer> {
    context: Context<'a, D, R>,
}

impl<'a, D, R> Plot<'a, D, R>
where
    D: Float,
    R: self::Renderer,
{
    pub fn new(
        tessellators: &'a mut Tessellators,
        renderer: &'a mut R,
        clip_bounds: &'a iced::Rectangle,
        mesh_buffer: &'a mut MeshBuffer,
        transform: &'a Transform<'a, D, f32, f32>,
    ) -> Self {
        renderer.start_layer(*clip_bounds);
        let context = Context {
            transform,
            clip_bounds,
            renderer,
            tessellators,
            mesh_buffer,
            last_drawn: ShapeType::Mesh,
        };
        Self { context }
    }

    pub fn bounds(&self) -> PlotRect<D> {
        self.context.transform.plot_bounds()
    }

    pub fn add_shape<S: Shape<D, R>>(&mut self, shape: S) {
        shape.render(&mut self.context)
    }

    // OLD CODE: See Context now
    // fn add_to_mesh(&mut self, shape: Shape<D>) {
    //     shape.add_to_buffer(self.transform, self.tessellators, self.mesh_buffer);
    //
    //     if self.mesh_buffer.vertices_count() >= self.mesh_buffer.limit() {
    //         self.mesh_buffer.render(self.renderer, self.bounds);
    //         self.renderer.end_layer();
    //         self.renderer.start_layer(*self.bounds);
    //     }
    // }
    //
    // fn render_text(&mut self, shape: Shape<D>) {
    //     // --- CRITICAL FIX FOR Z-ORDERING ---
    //     // This is an immediate-mode shape (like text).
    //     // We MUST render all meshes that came before it *first*.
    //
    //     // 1. Render all meshes currently in the buffer.
    //     self.mesh_buffer.render(self.renderer, self.bounds);
    //
    //     // 2. End the layer that contained those meshes.
    //     self.renderer.end_layer();
    //
    //     // 3. Start a NEW layer just for this single immediate shape.
    //     self.renderer.start_layer(*self.bounds);
    //
    //     // 4. Render the immediate shape (text, etc.).
    //     shape.render(self.transform, self.renderer);
    //
    //     // 5. End the layer for the immediate shape.
    //     self.renderer.end_layer();
    //
    //     // 6. Start another NEW layer for the *next* batch of meshes.
    //     self.renderer.start_layer(*self.bounds);
    // }
}

impl<'a, D, R> Drop for Plot<'a, D, R>
where
    D: Float,
    R: iced::advanced::Renderer
        + iced::advanced::graphics::mesh::Renderer
        + iced::advanced::text::Renderer<Font = iced::Font>,
{
    fn drop(&mut self) {
        self.context
            .mesh_buffer
            .render(self.context.renderer, self.context.clip_bounds);
        self.context.renderer.end_layer()
    }
}
