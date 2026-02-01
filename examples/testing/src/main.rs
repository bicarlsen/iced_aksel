use iced::alignment::{Horizontal, Vertical};
use iced::{
    Color, Element, Length, Subscription, Task, Theme,
    time::Instant,
    widget::{column, container, radio, row, scrollable, text},
    window,
};
use iced_aksel::stroke::StrokeStyle;
use iced_aksel::{
    Chart, Measure, PlotPoint, State, Stroke,
    axis::{self, Axis},
    plot::{Plot, PlotData},
    scale::Linear,
    shape::{
        Arc, Area, Bezier, Bounds, Ellipse, Label, Line, Polygon, Polyline, Rectangle, Spline,
        Triangle,
    },
};

// -----------------------------------------------------------------------------
// 1. Main Entry
// -----------------------------------------------------------------------------
pub fn main() -> iced::Result {
    iced::application(GalleryApp::default, GalleryApp::update, GalleryApp::view)
        .subscription(GalleryApp::subscription)
        .theme(Theme::Dark)
        .run()
}

// -----------------------------------------------------------------------------
// 2. Application State
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Rectangle,
    Ellipse,
    Triangle,
    Polygon,
    Line,
    Polyline,
    Spline,
    Bezier,
    Arc,
    Area,
    Label,
}

impl Page {
    fn all() -> [Page; 11] {
        [
            Page::Rectangle,
            Page::Ellipse,
            Page::Triangle,
            Page::Polygon,
            Page::Line,
            Page::Polyline,
            Page::Spline,
            Page::Bezier,
            Page::Arc,
            Page::Area,
            Page::Label,
        ]
    }
}

struct GalleryApp {
    chart_state: State<String, f64>,
    gallery_data: ShapeGallery,
    current_page: Page,
    last_frame_time: Option<Instant>,
    fps: f32,
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    PageSelected(Page),
}

impl Default for GalleryApp {
    fn default() -> Self {
        let mut state = State::new();

        // 60x60 grid for testing
        // Increased X slightly to make room for labels on the left
        state.set_axis(
            "x".to_string(),
            Axis::new(Linear::new(-10.0, 60.0), axis::Position::Bottom),
        );
        state.set_axis(
            "y".to_string(),
            Axis::new(Linear::new(0.0, 70.0), axis::Position::Left),
        );

        Self {
            chart_state: state,
            gallery_data: ShapeGallery {
                page: Page::Rectangle,
            },
            current_page: Page::Rectangle,
            last_frame_time: None,
            fps: 0.0,
        }
    }
}

impl GalleryApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                if let Some(last) = self.last_frame_time {
                    let delta = now.duration_since(last).as_secs_f32();
                    if delta > 0.0 {
                        self.fps = self.fps * 0.9 + (1.0 / delta) * 0.1;
                    }
                }
                self.last_frame_time = Some(now);
                Task::none()
            }
            Message::PageSelected(page) => {
                self.current_page = page;
                self.gallery_data.page = page;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- Sidebar ---
        let sidebar = column(Page::all().iter().map(|page| {
            radio(
                format!("{:?}", page),
                *page,
                Some(self.current_page),
                Message::PageSelected,
            )
            .into()
        }))
        .spacing(10)
        .padding(10);

        // --- Header ---
        let header = row![
            container(text(format!("FPS: {:.0}", self.fps)).size(16))
                .padding(5)
                .style(container::rounded_box),
            text(format!("Viewing: {:?}", self.current_page)).size(20),
            text("| Rows: Solid -> Dashed -> Dotted")
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7))
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);

        // --- Chart ---
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.gallery_data, "x".to_string(), "y".to_string())
            .width(Length::Fill)
            .height(Length::Fill)
            .damage(true);

        row![
            scrollable(sidebar).width(150),
            column![header, chart]
                .spacing(10)
                .padding(10)
                .width(Length::Fill)
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        window::frames().map(Message::Tick)
    }
}

// ----------------------------------------------------------------------------
// 3. Data Logic (The Gallery Implementation)
// ----------------------------------------------------------------------------

struct ShapeGallery {
    page: Page,
}

impl PlotData<f64> for ShapeGallery {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        // --- Define the 3 Variants ---
        let variants = [
            ("Solid", StrokeStyle::Solid, 55.0),
            (
                "Dashed",
                StrokeStyle::Dashed {
                    dash: 8.0,
                    gap: 4.0,
                },
                35.0,
            ),
            ("Dotted", StrokeStyle::Dotted { gap: 5.0 }, 15.0),
        ];

        // Draw Row Labels
        for (name, _, y_base) in &variants {
            plot.add_shape(
                Label::new(*name, PlotPoint::new(-8.0, *y_base))
                    .align(Horizontal::Left, Vertical::Center)
                    .size(Measure::Screen(14.0))
                    .fill(Color::WHITE),
            );
            // Divider line
            plot.add_shape(Line::new(
                PlotPoint::new(-10.0, y_base - 10.0),
                PlotPoint::new(60.0, y_base - 10.0),
                Stroke::new(Color::from_rgba(1.0, 1.0, 1.0, 0.1), Measure::Screen(1.0)),
            ));
        }

        // Delegate to specific shape drawers, passing the variant config
        match self.page {
            Page::Rectangle => self.draw_rectangles(plot, &variants),
            Page::Ellipse => self.draw_ellipses(plot, &variants),
            Page::Triangle => self.draw_triangles(plot, &variants),
            Page::Polygon => self.draw_polygons(plot, &variants),
            Page::Line => self.draw_lines(plot, &variants),
            Page::Polyline => self.draw_polylines(plot, &variants),
            Page::Spline => self.draw_splines(plot, &variants),
            Page::Bezier => self.draw_beziers(plot, &variants),
            Page::Arc => self.draw_arcs(plot, &variants),
            Page::Area => self.draw_areas(plot, &variants),
            Page::Label => self.draw_labels(plot, &variants),
        }
    }
}

type Variants = [(&'static str, StrokeStyle, f64); 3];

impl ShapeGallery {
    fn draw_rectangles(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // 1. Simple Stroke
            plot.add_shape(
                Rectangle::centered(
                    PlotPoint::new(10.0, *y_base),
                    Measure::Screen(30.0),
                    Measure::Screen(15.0),
                )
                .stroke(stroke),
            );

            // 2. Stroke + Fill
            plot.add_shape(
                Rectangle::corners(
                    PlotPoint::new(25.0, *y_base - 5.0),
                    PlotPoint::new(35.0, *y_base + 5.0),
                )
                .fill(Color::from_rgb(0.2, 0.2, 0.8))
                .stroke(stroke),
            );

            // 3. Thick Stroke
            plot.add_shape(
                Rectangle::centered(
                    PlotPoint::new(50.0, *y_base),
                    Measure::Screen(30.0),
                    Measure::Screen(15.0),
                )
                .stroke(Stroke::with_style(
                    Color::from_rgb(1.0, 0.5, 0.0),
                    Measure::Screen(5.0),
                    *style,
                )),
            );
        }
    }

    fn draw_ellipses(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // 1. Circle
            plot.add_shape(
                Ellipse::circle(PlotPoint::new(10.0, *y_base), Measure::Screen(10.0))
                    .stroke(stroke),
            );

            // 2. Filled Ellipse + Stroke
            plot.add_shape(
                Ellipse::new(
                    PlotPoint::new(30.0, *y_base),
                    Measure::Screen(15.0),
                    Measure::Screen(8.0),
                )
                .fill(Color::from_rgba(0.2, 0.8, 0.2, 0.5))
                .stroke(stroke),
            );

            // 3. Thick Stroke
            plot.add_shape(
                Ellipse::circle(PlotPoint::new(50.0, *y_base), Measure::Screen(10.0)).stroke(
                    Stroke::with_style(
                        Color::from_rgb(1.0, 0.2, 0.2),
                        Measure::Screen(4.0),
                        *style,
                    ),
                ),
            );
        }
    }

    fn draw_triangles(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // 1. Vertex Triangle
            plot.add_shape(
                Triangle::new(
                    PlotPoint::new(5.0, *y_base - 5.0),
                    PlotPoint::new(10.0, *y_base + 5.0),
                    PlotPoint::new(15.0, *y_base - 5.0),
                )
                .stroke(stroke),
            );

            // 2. Centered + Fill
            plot.add_shape(
                Triangle::centered(
                    PlotPoint::new(30.0, *y_base),
                    Measure::Screen(20.0),
                    Measure::Screen(15.0),
                )
                .fill(Color::from_rgb(0.2, 0.2, 0.8))
                .stroke(stroke),
            );
        }
    }

    fn draw_polygons(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // Hexagon
            plot.add_shape(
                Polygon::new(PlotPoint::new(15.0, *y_base), Measure::Screen(15.0), 6)
                    .stroke(stroke),
            );

            // Octagon Filled
            plot.add_shape(
                Polygon::new(PlotPoint::new(45.0, *y_base), Measure::Screen(15.0), 8)
                    .fill(Color::from_rgba(0.5, 0.0, 0.5, 0.5))
                    .stroke(Stroke::with_style(
                        Color::WHITE,
                        Measure::Screen(3.0),
                        *style,
                    )),
            );
        }
    }

    fn draw_lines(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // 1. Basic Line
            plot.add_shape(Line::new(
                PlotPoint::new(5.0, *y_base),
                PlotPoint::new(20.0, *y_base),
                stroke,
            ));

            // 2. Arrows (Start/End)
            plot.add_shape(
                Line::new(
                    PlotPoint::new(25.0, *y_base),
                    PlotPoint::new(40.0, *y_base),
                    stroke,
                )
                .arrow_end(true),
            );

            // 3. Diagonal
            plot.add_shape(
                Line::new(
                    PlotPoint::new(45.0, *y_base - 5.0),
                    PlotPoint::new(55.0, *y_base + 5.0),
                    Stroke::with_style(Color::WHITE, Measure::Screen(3.0), *style),
                )
                .arrows(true),
            );
        }
    }

    fn draw_polylines(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            let points = vec![
                PlotPoint::new(10.0, *y_base - 5.0),
                PlotPoint::new(20.0, *y_base + 5.0),
                PlotPoint::new(30.0, *y_base - 5.0),
                PlotPoint::new(40.0, *y_base + 5.0),
                PlotPoint::new(50.0, *y_base - 5.0),
            ];

            plot.add_shape(Polyline::new(points, stroke));
        }
    }

    fn draw_splines(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            let points = vec![
                PlotPoint::new(10.0, *y_base),
                PlotPoint::new(20.0, *y_base + 8.0),
                PlotPoint::new(30.0, *y_base - 8.0),
                PlotPoint::new(40.0, *y_base + 8.0),
                PlotPoint::new(50.0, *y_base),
            ];

            // 1. Standard Spline
            plot.add_shape(Spline::new(points, stroke).tension(0.0));
        }
    }

    fn draw_beziers(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // 1. Quadratic
            plot.add_shape(Bezier::quadratic(
                PlotPoint::new(10.0, *y_base),
                PlotPoint::new(20.0, *y_base + 15.0),
                PlotPoint::new(30.0, *y_base),
                stroke,
            ));

            // 2. Cubic
            plot.add_shape(Bezier::cubic(
                PlotPoint::new(35.0, *y_base),
                PlotPoint::new(45.0, *y_base + 15.0),
                PlotPoint::new(45.0, *y_base - 15.0),
                PlotPoint::new(55.0, *y_base),
                Stroke::with_style(Color::WHITE, Measure::Screen(3.0), *style),
            ));
        }
    }

    fn draw_arcs(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // 1. Pie Slice Outline
            plot.add_shape(
                Arc::new(
                    PlotPoint::new(15.0, *y_base),
                    Measure::Screen(20.0),
                    0.0,
                    1.57, // 90 deg
                )
                .stroke(stroke),
            );

            // 2. Donut Outline
            plot.add_shape(
                Arc::new(
                    PlotPoint::new(45.0, *y_base),
                    Measure::Screen(20.0),
                    0.0,
                    4.0,
                )
                .inner_radius(Measure::Screen(10.0))
                .stroke(Stroke::with_style(
                    Color::WHITE,
                    Measure::Screen(4.0),
                    *style,
                )),
            );
        }
    }

    fn draw_areas(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // Area with stroke outline
            let points = vec![
                PlotPoint::new(20.0, *y_base - 8.0),
                PlotPoint::new(30.0, *y_base + 8.0),
                PlotPoint::new(40.0, *y_base - 8.0),
                PlotPoint::new(20.0, *y_base - 8.0),
            ];

            plot.add_shape(
                Area::new(points)
                    .fill(Color::from_rgba(0.5, 0.5, 1.0, 0.3))
                    .stroke(stroke),
            );
        }
    }

    fn draw_labels(&self, plot: &mut Plot<f64>, variants: &Variants) {
        for (_, style, y_base) in variants {
            let stroke = Stroke::with_style(Color::WHITE, Measure::Screen(1.0), *style);

            // 1. Label inside a box
            // Note: Labels themselves don't stroke the font glyphs in this engine,
            // but we can draw a stroked box *around* the label to show the style.
            let x = 30.0;

            plot.add_shape(
                Rectangle::centered(
                    PlotPoint::new(x, *y_base),
                    Measure::Screen(100.0),
                    Measure::Screen(25.0),
                )
                .stroke(stroke),
            );

            plot.add_shape(
                Label::new("Stroked Box", PlotPoint::new(x, *y_base))
                    .align(Horizontal::Center, Vertical::Center)
                    .fill(Color::WHITE),
            );
        }
    }
}
