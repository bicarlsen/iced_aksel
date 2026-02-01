use lyon_path::builder::PathBuilder;
use lyon_path::iterator::PathIterator;
use lyon_path::{Event, Path};

/// Holds the mutable state required to generate a dashed path.
struct DashState<'a> {
    builder: lyon_path::path::Builder,
    dashes: &'a [f32],
    dash_index: usize,
    remaining_dash_len: f32,
    is_drawing: bool,
    is_subpath_open: bool,
}

impl<'a> DashState<'a> {
    fn new(dashes: &'a [f32]) -> Self {
        Self {
            builder: Path::builder(),
            dashes,
            dash_index: 0,
            remaining_dash_len: dashes[0],
            is_drawing: true,
            is_subpath_open: false,
        }
    }

    /// Resets the dash pattern state (called when a new contour begins).
    fn reset_pattern(&mut self) {
        self.dash_index = 0;
        self.remaining_dash_len = self.dashes[0];
        self.is_drawing = true;
    }

    /// Explicitly closes the current sub-path if it is open.
    /// This creates a gap in the geometry.
    fn close_gap(&mut self) {
        if self.is_subpath_open {
            self.builder.end(false);
            self.is_subpath_open = false;
        }
    }

    /// Processes a linear segment, creating dashes/gaps along it.
    fn process_segment(&mut self, p1: lyon::math::Point, p2: lyon::math::Point) {
        let vec = p2 - p1;
        let len = vec.length();

        // Avoid degenerate segments
        if len < 1e-6 {
            return;
        }

        let dir = vec / len;
        let mut dist_traveled = 0.0;
        let mut current_pos = p1;

        while dist_traveled < len {
            let dist_left = len - dist_traveled;
            let step = f32::min(self.remaining_dash_len, dist_left);
            let next_pos = current_pos + (dir * step);

            if self.is_drawing {
                // If we are drawing but don't have an open sub-path, start one.
                if !self.is_subpath_open {
                    self.builder.begin(current_pos);
                    self.is_subpath_open = true;
                }
                self.builder.line_to(next_pos);
            } else {
                // If we are in a gap, ensure the previous drawing path is closed.
                if self.is_subpath_open {
                    self.builder.end(false);
                    self.is_subpath_open = false;
                }
            }

            // Advance state
            current_pos = next_pos;
            dist_traveled += step;
            self.remaining_dash_len -= step;

            // Cycle pattern if step finished
            if self.remaining_dash_len <= 1e-5 {
                self.dash_index = (self.dash_index + 1) % self.dashes.len();
                self.remaining_dash_len = self.dashes[self.dash_index];
                self.is_drawing = !self.is_drawing;
            }
        }
    }
}

/// Creates a new path by dashing the source path.
pub fn create_dashed_path(source: &Path, dashes: &[f32], _phase: f32) -> Path {
    let flattened = source.iter().flattened(0.05);

    // Create the state machine wrapper
    let mut state = DashState::new(dashes);

    let mut current_pen_pos = lyon::math::point(0.0, 0.0);
    let mut contour_start_pos = lyon::math::point(0.0, 0.0);

    for event in flattened {
        match event {
            Event::Begin { at } => {
                // Clean up previous contour if needed
                state.close_gap();

                contour_start_pos = at;
                current_pen_pos = at;

                state.reset_pattern();

                // Note: We don't explicitly call builder.begin() here.
                // process_segment handles it lazily when it needs to draw.
            }
            Event::Line { to, .. } => {
                state.process_segment(current_pen_pos, to);
                current_pen_pos = to;
            }
            Event::End { close, .. } => {
                // If the path is closed, draw the connecting line
                if close {
                    state.process_segment(current_pen_pos, contour_start_pos);
                }

                // Always ensure the final dash is closed cleanly
                state.close_gap();
            }
            _ => {} // Quadratic/Cubic events are filtered by .flattened()
        }
    }

    // Final safety check
    state.close_gap();
    state.builder.build()
}
