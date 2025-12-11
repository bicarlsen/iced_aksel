use iced::{Color, Padding, Pixels, Theme};

/// Style of a `Chart`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub cursor: PlotCursor,
    pub axis: AxisStyle,
    pub grid_color: Color,
}

/// Style of a `Chart`s axis. This defines how the axis is rendering with the Chart
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisStyle {
    pub cursor: AxisCursor,
    pub tick_color: Color,
    pub label_color: Color,
}

/// Style of a `Chart`s axis cursor. This defines how the position of your mouse is rendering on the Axis
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisCursor {
    pub color: Color,
    pub width: Pixels,
}

/// TODO: Comment not sure what it does
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotCursor {
    pub color: Color,
    pub width: Pixels,
}

/// TODO: Comment not sure how to explain what it does
pub trait Catalog {
    type Class<'a>;

    /// TODO: Comment not sure how to explain what it does
    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    /// TODO: Comment not sure how to explain what it does
    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style;
}

/// TODO: Comment not sure how to explain what it does
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    /// TODO: Comment not sure how to explain what it does
    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    /// TODO: Comment not sure how to explain what it does
    fn style(&self, class: &StyleFn<'_, Self>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        cursor: PlotCursor {
            color: palette.background.weaker.color,
            width: 1.0.into(),
        },
        axis: AxisStyle {
            cursor: AxisCursor {
                color: palette.background.weak.color,
                width: 10.0.into(),
            },
            tick_color: palette.background.weak.color,
            label_color: palette.background.weak.text,
        },
        grid_color: palette.background.weak.color,
    }
}
