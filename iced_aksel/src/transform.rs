use std::ops::{Add, Mul, Sub};

use iced::{Point, Rectangle};

use crate::widget::chart::Scale;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PlotPoint<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T> PlotPoint<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn random(bounds: PlotRectangle<T>) -> Self
    where
        T: Mul<f32, Output = T> + Copy + Add<Output = T>,
    {
        let x = bounds.x + (bounds.width * rand::random::<f32>());
        let y = bounds.y + (bounds.height * rand::random::<f32>());
        Self { x, y }
    }
}

impl rstar::Point for PlotPoint {
    const DIMENSIONS: usize = 2;
    type Scalar = f32;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Self {
            x: generator(0),
            y: generator(1),
        }
    }

    fn nth(&self, index: usize) -> f32 {
        match index {
            0 => self.x,
            1 => self.y,
            _ => panic!("Invalid dimension index"),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut f32 {
        match index {
            0 => &mut self.x as &mut f32,
            1 => &mut self.y as &mut f32,
            _ => panic!("Invalid dimension index"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PlotRectangle<T = f32> {
    x: T,
    y: T,
    width: T,
    height: T,
}

impl PlotRectangle<f32> {
    pub fn from_top_left(top_left: PlotPoint, width: f32, height: f32) -> Self {
        Self {
            x: top_left.x,
            y: top_left.y,
            width,
            height,
        }
    }

    pub fn from_points(top_left: PlotPoint, bottom_right: PlotPoint) -> Self {
        Self {
            x: top_left.x,
            y: top_left.y,
            width: bottom_right.x.sub(top_left.x).abs(),
            height: top_left.y.sub(bottom_right.y).abs(),
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn top_left(&self) -> PlotPoint {
        PlotPoint::new(self.x, self.y)
    }

    pub fn bot_right(&self) -> PlotPoint {
        PlotPoint::new(self.x + self.width, self.y + self.height)
    }

    pub fn min_x(&self) -> f32 {
        self.x
    }

    pub fn max_x(&self) -> f32 {
        self.x + self.width
    }

    pub fn min_y(&self) -> f32 {
        self.y
    }

    pub fn max_y(&self) -> f32 {
        self.y + self.height
    }

    pub fn contains(&self, point: &PlotPoint) -> bool {
        todo!()
    }

    pub fn contains_area(&self, area: &PlotRectangle) -> bool {
        self.min_x() <= area.min_x()
            && area.max_x() <= self.max_x()
            && self.min_y() <= area.min_y()
            && area.max_y() <= self.max_y()
    }
}

impl From<Rectangle> for PlotRectangle {
    fn from(value: Rectangle) -> Self {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = value;
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
// --- This is your Transform struct ---

/// Holds the relationship between two coordinate systems (rectangles)
/// and can transform points between them.
#[derive(Clone, Copy)]
pub struct Transform<'a, T = f32> {
    screen_rect: Rectangle,
    x_scale: &'a dyn Scale<Domain = T>,
    y_scale: &'a dyn Scale<Domain = T>,
}

impl<'a> Transform<'a> {
    /// Creates a new Transform mapping a screen rectangle to a chart rectangle.
    pub const fn new(
        screen_rect: Rectangle,
        x_scale: &'a dyn Scale<Domain = f32>,
        y_scale: &'a dyn Scale<Domain = f32>,
    ) -> Self {
        Self {
            screen_rect,
            x_scale,
            y_scale,
        }
    }
}

/// Implementation of the transformation logic.
impl<'a> Transform<'a> {
    pub const fn screen_bounds(&self) -> &Rectangle {
        &self.screen_rect
    }

    /// Transforms a point from screen coordinates to chart coordinates.
    pub fn screen_to_chart(&self, screen_point: Point) -> PlotPoint {
        // Use the new single-axis helpers
        let cx = self.x_from_screen(screen_point.x);
        let cy = self.y_from_screen(screen_point.y);

        PlotPoint::new(cx, cy)
    }

    /// Transforms a point from chart coordinates to screen coordinates.
    pub fn chart_to_screen(&self, plot_point: PlotPoint) -> Point {
        // Use the new single-axis helpers
        let sx = self.x_to_screen(plot_point.x);
        let sy = self.y_to_screen(plot_point.y);

        Point::new(sx, sy)
    }

    // --- NEW: Single-Axis Helper Functions ---

    /// Transforms a single chart x-coordinate to a screen x-coordinate.
    pub fn x_to_screen(&self, plot_x: f32) -> f32 {
        let tx = self.x_scale.normalize(plot_x);
        (tx).mul_add(self.screen_rect.width, self.screen_rect.x)
    }

    /// Transforms a single chart y-coordinate to a screen y-coordinate.
    /// (This includes the y-axis inversion)
    pub fn y_to_screen(&self, plot_y: f32) -> f32 {
        let ty = self.y_scale.normalize(plot_y);
        // (1.0 - ty) inverts the coordinate
        (1.0 - ty).mul_add(self.screen_rect.height, self.screen_rect.y)
    }

    /// Transforms a single screen x-coordinate to a chart x-coordinate.
    pub fn x_from_screen(&self, screen_x: f32) -> f32 {
        let tx = (screen_x - self.screen_rect.x) / self.screen_rect.width;
        self.x_scale.denormalize(tx)
    }

    /// Transforms a single screen y-coordinate to a chart y-coordinate.
    /// (This accounts for the y-axis inversion)
    pub fn y_from_screen(&self, screen_y: f32) -> f32 {
        let ty_raw = (screen_y - self.screen_rect.y) / self.screen_rect.height;
        let ty = 1.0 - ty_raw; // Invert the normalized screen Y
        self.y_scale.denormalize(ty)
    }

    // --- UPDATED: Rect transformations (using new helpers) ---

    /// Transforms a rectangle from screen coordinates to chart coordinates.
    pub fn screen_to_chart_rect(&self, screen_rect: Rectangle) -> PlotRectangle {
        // 1. Get screen corner points
        let top_left_screen = Point::new(screen_rect.x, screen_rect.y);
        let bottom_right_screen = Point::new(
            screen_rect.x + screen_rect.width,
            screen_rect.y + screen_rect.height,
        );

        // 2. Transform corner points to chart space
        let top_left_chart = self.screen_to_chart(top_left_screen);
        let bottom_right_chart = self.screen_to_chart(bottom_right_screen);

        // 3. Reconstruct the rectangle in chart space
        // (Handle potential inverted Y)
        let x = top_left_chart.x;
        let y = top_left_chart.y; // Y is now max Y (top)
        let width = bottom_right_chart.x - top_left_chart.x;
        let height = bottom_right_chart.y - top_left_chart.y; // This will be negative

        PlotRectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Transforms a rectangle from chart coordinates to screen coordinates.
    pub fn chart_to_screen_rect(&self, chart_rect: PlotRectangle) -> Rectangle {
        // 1. Get chart corner points
        // We must normalize manually to handle negative height
        let (y_min, y_max) = if chart_rect.height < 0.0 {
            (chart_rect.y + chart_rect.height, chart_rect.y)
        } else {
            (chart_rect.y, chart_rect.y + chart_rect.height)
        };

        let (x_min, x_max) = if chart_rect.width < 0.0 {
            (chart_rect.x + chart_rect.width, chart_rect.x)
        } else {
            (chart_rect.x, chart_rect.x + chart_rect.width)
        };

        // 2. Transform corner points to screen space
        // Note: chart_to_screen maps (max_y) -> (top_screen)
        let top_left_screen = self.chart_to_screen(PlotPoint::new(x_min, y_max));
        let bottom_right_screen = self.chart_to_screen(PlotPoint::new(x_max, y_min));

        // 3. Reconstruct the rectangle in screen space
        let screen_width = bottom_right_screen.x - top_left_screen.x;
        let screen_height = bottom_right_screen.y - top_left_screen.y;

        Rectangle {
            x: top_left_screen.x,
            y: top_left_screen.y,
            width: screen_width,
            height: screen_height,
        }
    }
}
