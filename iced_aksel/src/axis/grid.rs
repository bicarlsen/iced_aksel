use iced::Pixels;

#[derive(Debug, Clone, Copy)]
pub struct GridLine {
    pub thickness: Pixels,
}

impl Default for GridLine {
    fn default() -> Self {
        Self {
            thickness: Pixels(1.0),
        }
    }
}
