use lyon::math::{Point, point};
use lyon::path::PathEvent;

pub struct DashedPolyline<'a, I>
where
    I: Iterator<Item = Point>,
{
    input: I,
    pattern: &'a [f32],

    current_pos: Option<Point>,
    next_pos: Option<Point>,
    pattern_idx: usize,
    dash_remaining: f32,
    is_gap: bool,

    // We need a small buffer because one step can generate:
    // Begin -> Line -> End (3 events)
    pending_line: Option<PathEvent>,
    pending_end: Option<PathEvent>,
    finishing_segment: bool,
}

impl<'a, I> DashedPolyline<'a, I>
where
    I: Iterator<Item = Point>,
{
    pub fn new(mut input: I, pattern: &'a [f32]) -> Self {
        let current_pos = input.next();
        let next_pos = input.next();
        let first_dash = pattern.first().copied().unwrap_or(1.0);

        Self {
            input,
            pattern,
            current_pos,
            next_pos,
            pattern_idx: 0,
            dash_remaining: first_dash,
            is_gap: false,
            pending_line: None,
            pending_end: None,
            finishing_segment: false,
        }
    }
}

impl<'a, I> Iterator for DashedPolyline<'a, I>
where
    I: Iterator<Item = Point>,
{
    type Item = PathEvent;

    fn next(&mut self) -> Option<Self::Item> {
        // 1. Flush Pending Events (Ordered: Line then End)
        if let Some(ev) = self.pending_line.take() {
            return Some(ev);
        }
        if let Some(ev) = self.pending_end.take() {
            return Some(ev);
        }

        // 2. Handle End-of-Stream
        //
        // TODO: Dennis - Ved ikke om det er dig der har lavet det her,
        // men bare til fremtiden, kan det være mere clean og forståeligt (Se under kommentaren)
        //
        // if self.current_pos.is_none() {
        //     if self.finishing_segment {
        //         self.finishing_segment = false;
        //         return Some(PathEvent::End {
        //             first: point(0.0, 0.0),
        //             last: point(0.0, 0.0),
        //             close: false,
        //         });
        //     }
        //     return None;
        // }
        //
        // let start = self.current_pos.unwrap();
        //
        // if self.next_pos.is_none() {
        //     self.current_pos = None;
        //     return self.next();
        // }
        //
        // let end = self.next_pos.unwrap();

        let Some(start) = self.current_pos else {
            if self.finishing_segment {
                self.finishing_segment = false;
                return Some(PathEvent::End {
                    first: point(0.0, 0.0),
                    last: point(0.0, 0.0),
                    close: false,
                });
            }
            return None;
        };

        let Some(end) = self.next_pos else {
            self.current_pos = None;
            return self.next();
        };

        // 3. Vector Math
        let delta = end - start;
        let dist = delta.length();

        if dist < 1e-6 {
            self.current_pos = self.next_pos;
            self.next_pos = self.input.next();
            return self.next();
        }

        // 4. Calculate Step
        let take = dist.min(self.dash_remaining);

        let end_of_step = if take >= dist {
            end
        } else {
            start + delta * (take / dist)
        };

        // 5. Generate Primary Event (Begin or Line)
        let event = if self.is_gap {
            None
        } else if self.finishing_segment {
            // We are already drawing, so the next event is the Line itself
            Some(PathEvent::Line {
                from: start,
                to: end_of_step,
            })
        } else {
            // We are starting a dash.
            // Immediate event: Begin.
            // Buffered event: Line.
            self.pending_line = Some(PathEvent::Line {
                from: start,
                to: end_of_step,
            });
            self.finishing_segment = true;
            Some(PathEvent::Begin { at: start })
        };

        // 6. Advance State
        self.dash_remaining -= take;
        self.current_pos = Some(end_of_step);

        // 7. Handle Pattern Completion (Dash <-> Gap)
        if self.dash_remaining <= 1e-5 {
            let was_dash = !self.is_gap;

            self.is_gap = !self.is_gap;
            self.pattern_idx = (self.pattern_idx + 1) % self.pattern.len();
            self.dash_remaining = self.pattern[self.pattern_idx];

            if was_dash {
                // We finished a dash. We must emit End.
                // We buffer it in `pending_end` so we don't overwrite `pending_line`.
                self.pending_end = Some(PathEvent::End {
                    first: point(0.0, 0.0),
                    last: point(0.0, 0.0),
                    close: false,
                });
                self.finishing_segment = false;
            }
        }

        // 8. Handle Geometric Segment Completion
        if take >= dist {
            self.current_pos = self.next_pos;
            self.next_pos = self.input.next();
        }

        // 9. Recurse (handle gap skipping or buffer flushing)
        if event.is_none() {
            return self.next();
        }

        event
    }
}
