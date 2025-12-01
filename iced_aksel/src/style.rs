use iced::{Color, Pixels, Theme};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub background: Color,
    pub cursor: PlotCursor,
    pub axis: AxisStyle,
    pub grid_color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisStyle {
    pub background: Color,
    pub cursor: AxisCursor,
    pub tick_color: Color,
    pub label_color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisCursor {
    pub color: Color,
    pub width: Pixels,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotCursor {
    pub color: Color,
    pub width: Pixels,
}

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> <Self as Catalog>::Class<'a>;

    fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> StyleFn<'a, Self> {
        Box::new(default)
    }

    fn style(&self, class: &StyleFn<'_, Self>) -> Style {
        class(self)
    }
}

pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        background: palette.background.weakest.color,
        cursor: PlotCursor {
            color: palette.background.weaker.color,
            width: 1.0.into(),
        },
        axis: AxisStyle {
            background: palette.background.weakest.color,
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
