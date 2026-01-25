pub enum Buffer<Renderer: crate::plot::Renderer> {
    Path(iced_graphics::geometry::Frame<Renderer>),
    Mesh(),
}

pub struct PrimitiveRenderer<Renderer: crate::plot::Renderer> {
    buffer: Buffer<Renderer>,
    quality: f32,
}

impl<Renderer> PrimitiveRenderer<Renderer> {}
