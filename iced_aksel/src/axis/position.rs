#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Top,
    Bottom,
    Left,
    Right,
}

impl From<Position> for Orientation {
    fn from(value: Position) -> Self {
        match value {
            Position::Top | Position::Bottom => Self::Horizontal,
            Position::Left | Position::Right => Self::Vertical,
        }
    }
}

impl<'a> From<&'a Position> for Orientation {
    fn from(value: &'a Position) -> Self {
        match value {
            Position::Top | Position::Bottom => Self::Horizontal,
            Position::Left | Position::Right => Self::Vertical,
        }
    }
}
