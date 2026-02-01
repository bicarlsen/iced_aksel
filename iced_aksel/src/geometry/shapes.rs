use iced_core::Point;

pub struct Rectangle {
    pub xy1: Point,
    pub xy2: Point,
}

impl Rectangle {
    pub fn new(xy1: Point, xy2: Point) -> Self {
        Self { xy1, xy2 }
    }
}
