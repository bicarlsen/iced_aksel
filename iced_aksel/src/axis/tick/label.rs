use iced::Pixels;

#[derive(Debug, Clone)]
pub struct Label {
    pub size: Pixels,
    pub content: String,
}

impl Default for Label {
    #[inline(always)]
    fn default() -> Self {
        Self {
            size: Pixels(12.0),
            content: String::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LabelBounds {
    pub start: f32,
    pub end: f32,
}

impl LabelBounds {
    pub const fn new(start: f32, end: f32) -> Self {
        Self { start, end }
    }

    pub fn overlaps_with_gap(&self, other: &Self, min_gap: f32) -> bool {
        (self.start < other.end + min_gap) && (other.start < self.end + min_gap)
    }
}
