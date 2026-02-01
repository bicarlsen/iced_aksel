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
        Arc,
        Area,
        Bezier,
        Bounds, // For Label bounds
        Ellipse,
        Label,
        Line,
        Polygon,
        Polyline,
        Rectangle,
        Spline,
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
        state.set_axis(
            "x".to_string(),
            Axis::new(Linear::new(0.0, 60.0), axis::Position::Bottom),
        );
        state.set_axis(
            "y".to_string(),
            Axis::new(Linear::new(0.0, 60.0), axis::Position::Left),
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
            text(format!("Viewing: {:?}", self.current_page)).size(20)
        ]
        .spacing(20)
        .align_y(iced::Alignment::Center);

        // --- Chart ---
        let chart = Chart::new(&self.chart_state)
            .plot_data(&self.gallery_data, "x".to_string(), "y".to_string())
            .width(Length::Fill)
            .height(Length::Fill);

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

// -----------------------------------------------------------------------------
// 3. Data Logic (The Gallery Implementation)
// -----------------------------------------------------------------------------

struct ShapeGallery {
    page: Page,
}

impl PlotData<f64> for ShapeGallery {
    fn draw(&self, plot: &mut Plot<f64>, _theme: &Theme) {
        match self.page {
            Page::Rectangle => self.draw_rectangles(plot),
            Page::Ellipse => self.draw_ellipses(plot),
            Page::Triangle => self.draw_triangles(plot),
            Page::Polygon => self.draw_polygons(plot),
            Page::Line => self.draw_lines(plot),
            Page::Polyline => self.draw_polylines(plot),
            Page::Spline => self.draw_splines(plot),
            Page::Bezier => self.draw_beziers(plot),
            Page::Arc => self.draw_arcs(plot),
            Page::Area => self.draw_areas(plot),
            Page::Label => self.draw_labels(plot),
        }
    }
}

impl ShapeGallery {
    fn draw_rectangles(&self, plot: &mut Plot<f64>) {
        // Row 1: Fill Corner-to-Corner
        for i in 0..5 {
            let x = 5.0 + (i as f64 * 10.0);
            plot.add_shape(
                Rectangle::corners(PlotPoint::new(x, 50.0), PlotPoint::new(x + 5.0, 55.0))
                    .fill(Color::from_rgb(0.2, 0.6, 1.0)),
            );
        }
        // Row 2: Stroke Centered
        for i in 0..5 {
            let x = 7.5 + (i as f64 * 10.0);
            plot.add_shape(
                Rectangle::centered(
                    PlotPoint::new(x, 35.0),
                    Measure::Screen(20.0),
                    Measure::Screen(20.0),
                )
                .stroke(Stroke::new(
                    Color::from_rgb(1.0, 0.8, 0.2),
                    Measure::Screen(2.0),
                )),
            );
        }
    }

    fn draw_ellipses(&self, plot: &mut Plot<f64>) {
        // Row 1: Circles (Fill)
        for i in 0..5 {
            let x = 7.5 + (i as f64 * 10.0);
            plot.add_shape(
                Ellipse::circle(PlotPoint::new(x, 50.0), Measure::Screen(15.0))
                    .fill(Color::from_rgba(0.8, 0.2, 0.8, 0.8)),
            );
        }
        // Row 2: Stretched Ellipses (Stroke)
        for i in 0..5 {
            let x = 7.5 + (i as f64 * 10.0);
            plot.add_shape(
                Ellipse::new(
                    PlotPoint::new(x, 30.0),
                    Measure::Plot(4.0), // Wide in Plot Units
                    Measure::Plot(1.5), // Short in Plot Units
                )
                .stroke(Stroke::new(Color::WHITE, Measure::Screen(2.0))),
            );
        }
    }

    fn draw_triangles(&self, plot: &mut Plot<f64>) {
        // Row 1: Defined by Vertices
        plot.add_shape(
            Triangle::new(
                PlotPoint::new(5.0, 50.0),
                PlotPoint::new(10.0, 55.0),
                PlotPoint::new(15.0, 50.0),
            )
            .fill(Color::from_rgb(0.2, 1.0, 0.2)),
        );

        // Row 2: Centered (Upward pointing markers)
        for i in 0..5 {
            let x = 7.5 + (i as f64 * 10.0);
            plot.add_shape(
                Triangle::centered(
                    PlotPoint::new(x, 30.0),
                    Measure::Screen(20.0),
                    Measure::Screen(20.0),
                )
                .fill(Color::from_rgb(1.0, 0.5, 0.0)),
            );
        }
    }

    fn draw_polygons(&self, plot: &mut Plot<f64>) {
        let counts = [3, 4, 5, 6, 8];
        // Row 1: Shapes
        for (i, &v) in counts.iter().enumerate() {
            let x = 7.5 + (i as f64 * 10.0);
            plot.add_shape(
                Polygon::new(PlotPoint::new(x, 40.0), Measure::Screen(20.0), v)
                    .fill(Color::from_rgb(0.3, 0.3, 1.0))
                    .stroke(Stroke::new(Color::WHITE, Measure::Screen(1.0))),
            );
            // Label
            plot.add_shape(Label::new(format!("{} Sides", v), PlotPoint::new(x, 35.0)));
        }

        // Row 2: Rotated Hexagons
        for i in 0..5 {
            let x = 7.5 + (i as f64 * 10.0);
            let rot = i as f32 * 15.0; // 0, 15, 30...
            plot.add_shape(
                Polygon::new(PlotPoint::new(x, 20.0), Measure::Screen(20.0), 6)
                    .rotation(rot)
                    .stroke(Stroke::new(
                        Color::from_rgb(1.0, 0.0, 0.5),
                        Measure::Screen(2.0),
                    )),
            );
        }
    }

    fn draw_lines(&self, plot: &mut Plot<f64>) {
        let stroke = Stroke::new(Color::WHITE, Measure::Screen(2.0));

        // 1. Simple Segment
        plot.add_shape(Line::new(
            PlotPoint::new(5.0, 50.0),
            PlotPoint::new(25.0, 55.0),
            stroke,
        ));

        // 2. Arrows
        plot.add_shape(
            Line::new(
                PlotPoint::new(5.0, 40.0),
                PlotPoint::new(25.0, 40.0),
                stroke,
            )
            .arrow_end(true),
        );
        plot.add_shape(
            Line::new(
                PlotPoint::new(30.0, 40.0),
                PlotPoint::new(50.0, 40.0),
                stroke,
            )
            .arrows(true), // Both ends
        );

        // 3. Infinite Extensions (Dashed)
        let dashed = Stroke::with_style(
            Color::from_rgb(0.5, 0.5, 0.5),
            Measure::Screen(1.0),
            StrokeStyle::Dashed {
                dash: 5.0,
                gap: 5.0,
            },
        );
        plot.add_shape(
            Line::new(
                PlotPoint::new(20.0, 20.0),
                PlotPoint::new(40.0, 15.0),
                dashed,
            )
            .infinite(),
        );
        // Draw the "core" segment solid to show where the math starts
        plot.add_shape(Line::new(
            PlotPoint::new(20.0, 20.0),
            PlotPoint::new(40.0, 15.0),
            stroke,
        ));
    }

    fn draw_polylines(&self, plot: &mut Plot<f64>) {
        let points = vec![
            PlotPoint::new(5.0, 30.0),
            PlotPoint::new(15.0, 50.0),
            PlotPoint::new(25.0, 30.0),
            PlotPoint::new(35.0, 50.0),
            PlotPoint::new(45.0, 30.0),
        ];

        // 1. Standard Polyline
        plot.add_shape(Polyline::new(
            points.clone(),
            Stroke::new(Color::from_rgb(0.2, 1.0, 0.2), Measure::Screen(3.0)),
        ));

        // 2. Polyline with Arrows
        let points_low = vec![
            PlotPoint::new(5.0, 10.0),
            PlotPoint::new(15.0, 20.0),
            PlotPoint::new(25.0, 10.0),
        ];
        plot.add_shape(
            Polyline::new(
                points_low,
                Stroke::new(Color::from_rgb(1.0, 0.2, 0.2), Measure::Screen(2.0)),
            )
            .arrow_end(true),
        );
    }

    fn draw_splines(&self, plot: &mut Plot<f64>) {
        let points = vec![
            PlotPoint::new(5.0, 30.0),
            PlotPoint::new(15.0, 50.0),
            PlotPoint::new(25.0, 30.0),
            PlotPoint::new(35.0, 50.0),
            PlotPoint::new(45.0, 30.0),
        ];

        // 1. Smooth (Catmull-Rom) - Green
        plot.add_shape(
            Spline::new(
                points.clone(),
                Stroke::new(Color::from_rgb(0.2, 1.0, 0.2), Measure::Screen(3.0)),
            )
            .tension(0.0),
        );

        // 2. High Tension (Straighter) - Red, Dashed
        let dashed = Stroke::with_style(
            Color::from_rgb(1.0, 0.2, 0.2),
            Measure::Screen(2.0),
            StrokeStyle::Dashed {
                dash: 5.0,
                gap: 5.0,
            },
        );
        plot.add_shape(Spline::new(points, dashed).tension(0.8));
    }

    fn draw_beziers(&self, plot: &mut Plot<f64>) {
        // 1. Quadratic (1 Control Point)
        let start = PlotPoint::new(5.0, 10.0);
        let ctrl = PlotPoint::new(15.0, 50.0);
        let end = PlotPoint::new(25.0, 10.0);

        // Draw handles for visualization
        self.draw_handles(plot, start, ctrl, end);

        plot.add_shape(Bezier::quadratic(
            start,
            ctrl,
            end,
            Stroke::new(Color::WHITE, Measure::Screen(3.0)),
        ));

        // 2. Cubic (2 Control Points)
        let c_start = PlotPoint::new(30.0, 10.0);
        let c1 = PlotPoint::new(30.0, 50.0);
        let c2 = PlotPoint::new(50.0, 50.0);
        let c_end = PlotPoint::new(50.0, 10.0);

        self.draw_handles_cubic(plot, c_start, c1, c2, c_end);

        plot.add_shape(Bezier::cubic(
            c_start,
            c1,
            c2,
            c_end,
            Stroke::new(Color::from_rgb(0.2, 0.8, 1.0), Measure::Screen(3.0)),
        ));
    }

    fn draw_arcs(&self, plot: &mut Plot<f64>) {
        // 1. Pie Slice (Inner radius 0)
        plot.add_shape(
            Arc::new(
                PlotPoint::new(15.0, 40.0),
                Measure::Screen(30.0),
                0.0,  // 0 deg
                1.57, // 90 deg
            )
            .fill(Color::from_rgb(1.0, 0.5, 0.0)),
        );

        // 2. Donut (Inner radius > 0)
        plot.add_shape(
            Arc::new(
                PlotPoint::new(45.0, 40.0),
                Measure::Screen(30.0),
                0.0,
                4.0, // > 180 deg
            )
            .inner_radius(Measure::Screen(15.0))
            .fill(Color::from_rgb(0.2, 0.8, 0.2))
            .stroke(Stroke::new(Color::WHITE, Measure::Screen(2.0))),
        );
    }

    fn draw_areas(&self, plot: &mut Plot<f64>) {
        // A simple star-like polygon
        let points = vec![
            PlotPoint::new(30.0, 50.0),
            PlotPoint::new(35.0, 35.0),
            PlotPoint::new(50.0, 30.0),
            PlotPoint::new(35.0, 25.0),
            PlotPoint::new(30.0, 10.0),
            PlotPoint::new(25.0, 25.0),
            PlotPoint::new(10.0, 30.0),
            PlotPoint::new(25.0, 35.0),
        ];

        plot.add_shape(
            Area::new(points)
                .fill(Color::from_rgba(0.5, 0.2, 0.8, 0.5)) // Transparent purple
                .stroke(Stroke::new(Color::WHITE, Measure::Screen(2.0))),
        );
    }

    fn draw_labels(&self, plot: &mut Plot<f64>) {
        // 1. Alignment Grid
        let positions = [
            (10.0, 50.0, Horizontal::Left, Vertical::Top, "TL"),
            (30.0, 50.0, Horizontal::Center, Vertical::Top, "TC"),
            (50.0, 50.0, Horizontal::Right, Vertical::Top, "TR"),
            (10.0, 30.0, Horizontal::Left, Vertical::Center, "ML"),
            (30.0, 30.0, Horizontal::Center, Vertical::Center, "MC"),
            (50.0, 30.0, Horizontal::Right, Vertical::Center, "MR"),
            (10.0, 10.0, Horizontal::Left, Vertical::Bottom, "BL"),
            (30.0, 10.0, Horizontal::Center, Vertical::Bottom, "BC"),
            (50.0, 10.0, Horizontal::Right, Vertical::Bottom, "BR"),
        ];

        for (x, y, h, v, text) in positions {
            // Draw a dot to see the anchor point
            plot.add_shape(
                Ellipse::circle(PlotPoint::new(x, y), Measure::Screen(2.0)).fill(Color::WHITE),
            );

            plot.add_shape(
                Label::new(text, PlotPoint::new(x, y))
                    .align(h, v)
                    .size(Measure::Screen(16.0))
                    .fill(Color::WHITE),
            );
        }

        // 2. Rotation
        plot.add_shape(
            Label::new("Rotated", PlotPoint::new(30.0, 20.0))
                .rotation(0.78) // ~45 deg
                .size(Measure::Screen(20.0))
                .fill(Color::from_rgb(1.0, 1.0, 0.0)),
        );
    }

    // --- Helpers ---

    fn draw_handles(
        &self,
        plot: &mut Plot<f64>,
        start: PlotPoint<f64>,
        ctrl: PlotPoint<f64>,
        end: PlotPoint<f64>,
    ) {
        let style = Stroke::with_style(
            Color::from_rgba(1.0, 1.0, 1.0, 0.3),
            Measure::Screen(1.0),
            StrokeStyle::Dashed {
                dash: 2.0,
                gap: 2.0,
            },
        );
        plot.add_shape(Line::new(start, ctrl, style));
        plot.add_shape(Line::new(ctrl, end, style));
        plot.add_shape(Ellipse::circle(ctrl, Measure::Screen(3.0)).fill(Color::WHITE));
    }

    fn draw_handles_cubic(
        &self,
        plot: &mut Plot<f64>,
        start: PlotPoint<f64>,
        c1: PlotPoint<f64>,
        c2: PlotPoint<f64>,
        end: PlotPoint<f64>,
    ) {
        let style = Stroke::with_style(
            Color::from_rgba(1.0, 1.0, 1.0, 0.3),
            Measure::Screen(1.0),
            StrokeStyle::Dashed {
                dash: 2.0,
                gap: 2.0,
            },
        );
        plot.add_shape(Line::new(start, c1, style));
        plot.add_shape(Line::new(c1, c2, style));
        plot.add_shape(Line::new(c2, end, style));
        plot.add_shape(Ellipse::circle(c1, Measure::Screen(3.0)).fill(Color::WHITE));
        plot.add_shape(Ellipse::circle(c2, Measure::Screen(3.0)).fill(Color::WHITE));
    }
}
