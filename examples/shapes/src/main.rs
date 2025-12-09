use iced::{
    Color, Theme,
    alignment::{Horizontal, Vertical},
};
use iced_aksel::{
    Axis, Chart, Length, State,
    axis::{self, GridLine},
    plot::{Items, Plot},
};
use aksel::{PlotPoint, scale::Linear};

// Import all available shapes
use iced_aksel::Stroke;
use iced_aksel::shape::{
    Arc, Circle, Label, Line, Polygon, Polyline, Rectangle, Triangle
};

type AxisId = &'static str;

pub fn main() -> iced::Result {
    iced::application(ShapeGallery::new, ShapeGallery::update, ShapeGallery::view)
        .title("Shapes Gallery")
        .antialiasing(true)
        .run()
}

pub struct ShapeGallery {
    state: State<AxisId, f64>,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl ShapeGallery {
    const X: &'static str = "X";
    const Y: &'static str = "Y";

    pub fn new() -> (Self, iced::Task<Message>) {
        let mut state = State::new();

        // 1. Setup Canvas
        // Expanded Y-range to fit the extra row (down to -30)
        state.set_axis(
            Self::X,
            Axis::new(Linear::new(-15.0, 135.0), axis::Position::Bottom)
                .invisible()
        );
        
        state.set_axis(
            Self::Y,
            Axis::new(Linear::new(-30.0, 190.0), axis::Position::Left)
                .invisible()
        );

        (Self { state }, iced::Task::none())
    }

    pub fn update(&mut self, _message: Message) -> iced::Task<Message> {
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<Message> {
        Chart::new(&self.state)
            .layer(self, Self::X, Self::Y)
            .into()
    }
}

impl Items<f64> for ShapeGallery {
    fn draw(&self, plot: &mut Plot<f64, iced::Renderer>, theme: &Theme) {
        let palette = theme.palette();
        let text_color = palette.text;
        
        // --- Helpers ---
        let draw_row_label = |plot: &mut Plot<f64, iced::Renderer>, y: f64, title: &str| {
            plot.add_shape(
                Label::new(title, PlotPoint::new(-10.0, y))
                    .size(14.0)
                    .fill(text_color)
                    .align(Horizontal::Left, Vertical::Center)
            );
        };

        let draw_col_header = |plot: &mut Plot<f64, iced::Renderer>, x: f64, title: &str| {
            plot.add_shape(
                Label::new(title, PlotPoint::new(x, 180.0))
                    .size(12.0)
                    .fill(text_color)
                    .align(Horizontal::Center, Vertical::Bottom)
            );
        };

        // --- Column Headers ---
        draw_col_header(plot, 25.0, "Plot\nFilled");
        draw_col_header(plot, 50.0, "Plot\nStroked");
        draw_col_header(plot, 75.0, "Fixed Px\nFilled");
        draw_col_header(plot, 100.0, "Fixed Px\nStroked");

        // =========================================================
        //  ROW 1: RECTANGLES (Y = 160)
        // =========================================================
        let y = 160.0;
        draw_row_label(plot, y, "Rect");

        plot.add_shape(
            Rectangle::new(PlotPoint::new(25.0, y), Length::Plot(12.0), Length::Plot(12.0))
                .fill(palette.primary)
        );

        plot.add_shape(
            Rectangle::new(PlotPoint::new(50.0, y), Length::Plot(12.0), Length::Plot(12.0))
                .stroke(Stroke::new(palette.primary, Length::Screen(2.0)))
        );

        plot.add_shape(
            Rectangle::new(PlotPoint::new(75.0, y), Length::Screen(25.0), Length::Screen(25.0))
                .fill(palette.success)
        );

        plot.add_shape(
            Rectangle::new(PlotPoint::new(100.0, y), Length::Screen(25.0), Length::Screen(25.0))
                .stroke(Stroke::new(palette.success, Length::Screen(2.0)))
        );

        // =========================================================
        //  ROW 2: CIRCLES (Y = 135)
        // =========================================================
        let y = 135.0;
        draw_row_label(plot, y, "Circle");

        plot.add_shape(
            Circle::new(PlotPoint::new(25.0, y), Length::Plot(6.0))
                .fill(palette.primary)
        );

        plot.add_shape(
            Circle::new(PlotPoint::new(50.0, y), Length::Plot(6.0))
                .stroke(Stroke::new(palette.primary, Length::Screen(2.0)))
        );

        plot.add_shape(
            Circle::new(PlotPoint::new(75.0, y), Length::Screen(12.0))
                .fill(palette.success)
        );

        plot.add_shape(
            Circle::new(PlotPoint::new(100.0, y), Length::Screen(12.0))
                .stroke(Stroke::new(palette.success, Length::Screen(2.0)))
        );

        // =========================================================
        //  ROW 3: ARCS (Y = 110)
        // =========================================================
        let y = 110.0;
        draw_row_label(plot, y, "Arc");

        plot.add_shape(
            Arc::new(PlotPoint::new(25.0, y), Length::Plot(7.0), 0.0, 4.0)
                .fill(palette.primary)
        );

        plot.add_shape(
            Arc::new(PlotPoint::new(50.0, y), Length::Plot(7.0), 0.0, 4.0)
                .stroke(Stroke::new(palette.primary, Length::Screen(2.0)))
        );

        plot.add_shape(
            Arc::new(PlotPoint::new(75.0, y), Length::Screen(14.0), 0.0, 4.0)
                .fill(palette.success)
        );

        plot.add_shape(
            Arc::new(PlotPoint::new(100.0, y), Length::Screen(14.0), 0.0, 4.0)
                .stroke(Stroke::new(palette.success, Length::Screen(2.0)))
        );

        // =========================================================
        //  ROW 4: TRIANGLES (Y = 85)
        // =========================================================
        let y = 85.0;
        draw_row_label(plot, y, "Triangle");

        plot.add_shape(
            Triangle::new(
                PlotPoint::new(20.0, y - 5.0),
                PlotPoint::new(30.0, y - 5.0),
                PlotPoint::new(25.0, y + 5.0),
            ).fill(palette.primary)
        );
        
        plot.add_shape(
            Triangle::new(
                PlotPoint::new(45.0, y - 5.0),
                PlotPoint::new(55.0, y - 5.0),
                PlotPoint::new(50.0, y + 5.0),
            ).stroke(Stroke::new(palette.primary, Length::Screen(2.0)))
        );

        plot.add_shape(
            Triangle::equilateral(
                PlotPoint::new(75.0, y),
                Length::Screen(14.0)
            ).fill(palette.success)
        );

        plot.add_shape(
            Triangle::equilateral(
                PlotPoint::new(100.0, y),
                Length::Screen(14.0)
            ).stroke(Stroke::new(palette.success, Length::Screen(2.0)))
        );


        // =========================================================
        //  ROW 5: POLYGONS (Y = 60)
        // =========================================================
        let y = 60.0;
        draw_row_label(plot, y, "Polygon");

        let poly_pts = |cx: f64| vec![
            PlotPoint::new(cx, y + 5.0),
            PlotPoint::new(cx + 4.0, y + 2.0),
            PlotPoint::new(cx + 2.0, y - 5.0),
            PlotPoint::new(cx - 2.0, y - 5.0),
            PlotPoint::new(cx - 4.0, y + 2.0),
        ];

        plot.add_shape(Polygon::new(poly_pts(25.0)).fill(palette.primary));
        
        plot.add_shape(Polygon::new(poly_pts(50.0)).stroke(Stroke::new(palette.primary, Length::Screen(2.0))));

        plot.add_shape(Label::new("N/A", PlotPoint::new(75.0, y)).size(10.0).fill(Color::from_rgb(0.5,0.5,0.5)));
        plot.add_shape(Label::new("N/A", PlotPoint::new(100.0, y)).size(10.0).fill(Color::from_rgb(0.5,0.5,0.5)));

        // =========================================================
        //  ROW 6: LINES / POLYLINES (Y = 35)
        // =========================================================
        let y = 35.0;
        draw_row_label(plot, y, "Path");
        
        plot.add_shape(Label::new("N/A", PlotPoint::new(25.0, y)).size(10.0).fill(Color::from_rgb(0.5,0.5,0.5)));

        let zigzag = |cx: f64| vec![
            PlotPoint::new(cx - 5.0, y), 
            PlotPoint::new(cx - 2.0, y + 5.0), 
            PlotPoint::new(cx + 2.0, y - 5.0), 
            PlotPoint::new(cx + 5.0, y)
        ];
        
        plot.add_shape(
            Polyline::new(
                zigzag(50.0), 
                Stroke::new(palette.primary, Length::Plot(1.0))
            )
        );

        plot.add_shape(Label::new("N/A", PlotPoint::new(75.0, y)).size(10.0).fill(Color::from_rgb(0.5,0.5,0.5)));

        plot.add_shape(
            Polyline::new(
                zigzag(100.0), 
                Stroke::new(palette.success, Length::Screen(3.0))
            )
        );

        // =========================================================
        //  SEPARATOR
        // =========================================================
        
        let sep_y = 25.0;
        
        plot.add_shape(
            Line::new(
                PlotPoint::new(-10.0, sep_y), 
                PlotPoint::new(130.0, sep_y),
                Stroke::new(Color::from_rgb(0.5, 0.5, 0.5), Length::Screen(1.0))
            )
        );

        plot.add_shape(
             Label::new("Text size is always defined in Screen Pixels", PlotPoint::new(60.0, sep_y - 5.0))
                .size(10.0)
                .fill(text_color)
                .align(Horizontal::Center, Vertical::Top)
        );

        // =========================================================
        //  ROW 7: LABEL STYLE (Y = 10)
        // =========================================================
        let y = 10.0;
        draw_row_label(plot, y, "Lbl Style");

        // 1. Standard
        plot.add_shape(
            Label::new("Standard", PlotPoint::new(25.0, y))
                .fill(palette.text)
                .size(12.0)
        );
        plot.add_shape(Label::new("(Default)", PlotPoint::new(25.0, y - 8.0)).size(8.0).fill(text_color));

        // 2. Custom Color
        plot.add_shape(
            Label::new("Colored", PlotPoint::new(50.0, y))
                .fill(palette.primary)
                .size(12.0)
        );
        plot.add_shape(Label::new("(Color)", PlotPoint::new(50.0, y - 8.0)).size(8.0).fill(text_color));
        
        // 3. Small Size
        plot.add_shape(
            Label::new("Small", PlotPoint::new(75.0, y))
                .fill(palette.success)
                .size(8.0)
        );
        plot.add_shape(Label::new("(8px)", PlotPoint::new(75.0, y - 8.0)).size(8.0).fill(text_color));

        // 4. Large Size
        plot.add_shape(
            Label::new("Large", PlotPoint::new(100.0, y))
                .fill(palette.success)
                .size(20.0)
        );
        plot.add_shape(Label::new("(20px)", PlotPoint::new(100.0, y - 8.0)).size(8.0).fill(text_color));

        // =========================================================
        //  ROW 8: LABEL ALIGNMENT (Y = -15)
        // =========================================================
        let y = -15.0;
        draw_row_label(plot, y, "Lbl Align");

        // Helper to draw an anchor point
        let draw_anchor = |plot: &mut Plot<f64, iced::Renderer>, pt: PlotPoint| {
            plot.add_shape(
                Rectangle::new(pt, Length::Screen(4.0), Length::Screen(4.0))
                    .fill(Color::from_rgb(1.0, 0.2, 0.2)) // Red anchor
            );
        };

        // 1. Top-Left
        let pt1 = PlotPoint::new(25.0, y);
        draw_anchor(plot, pt1);
        plot.add_shape(
            Label::new("Top\nLeft", pt1)
                .fill(palette.text)
                .size(10.0)
                .align(Horizontal::Left, Vertical::Top)
        );

        // 2. Center-Center
        let pt2 = PlotPoint::new(50.0, y);
        draw_anchor(plot, pt2);
        plot.add_shape(
            Label::new("Center", pt2)
                .fill(palette.text)
                .size(10.0)
                .align(Horizontal::Center, Vertical::Center)
        );

        // 3. Bottom-Right
        let pt3 = PlotPoint::new(75.0, y);
        draw_anchor(plot, pt3);
        plot.add_shape(
            Label::new("Bottom\nRight", pt3)
                .fill(palette.text)
                .size(10.0)
                .align(Horizontal::Right, Vertical::Bottom)
        );

        // 4. Bottom-Center
        let pt4 = PlotPoint::new(100.0, y);
        draw_anchor(plot, pt4);
        plot.add_shape(
            Label::new("Standing", pt4)
                .fill(palette.text)
                .size(10.0)
                .align(Horizontal::Center, Vertical::Bottom)
        );
    }
}