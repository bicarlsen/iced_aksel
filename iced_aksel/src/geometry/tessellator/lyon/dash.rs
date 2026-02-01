use lyon::math::{Point, Vector};
use lyon::path::PathEvent;

/// An iterator that turns a stream of FLATTENED Line segments into a dashed pattern.
pub struct DashingIterator<I> {
    iter: I,

    // Config
    dash_array: Vec<f32>,
    dash_index: usize,

    // State
    current_segment_len: f32,
    is_gap: bool,
    is_drawing: bool,

    // Geometry
    current_pos: Point,
    target_pos: Point,

    // Queue
    pending_event: Option<PathEvent>,
    closing_event: Option<PathEvent>,
}

impl<I> DashingIterator<I>
where
    I: Iterator<Item = PathEvent>,
{
    pub fn new(iter: I, dash_array: Vec<f32>) -> Self {
        let first_dash = if !dash_array.is_empty() {
            dash_array[0]
        } else {
            0.0
        };
        Self {
            iter,
            dash_array,
            dash_index: 0,
            current_segment_len: first_dash,
            is_gap: false,
            is_drawing: false,
            current_pos: Point::new(0.0, 0.0),
            target_pos: Point::new(0.0, 0.0),
            pending_event: None,
            closing_event: None,
        }
    }

    fn advance_dash_state(&mut self) {
        if self.dash_array.is_empty() {
            return;
        }
        self.dash_index = (self.dash_index + 1) % self.dash_array.len();
        self.current_segment_len = self.dash_array[self.dash_index];
        self.is_gap = !self.is_gap;
    }
}

impl<I> Iterator for DashingIterator<I>
where
    I: Iterator<Item = PathEvent>,
{
    type Item = PathEvent;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // 1. Flush pending events
            if let Some(event) = self.pending_event.take() {
                return Some(event);
            }

            // 2. Fetch or Switch Target
            if self.current_pos == self.target_pos {
                // A. Handle Closing Loop
                if let Some(end_event) = self.closing_event.take() {
                    // FIX: Only emit the closing End if we are currently drawing.
                    // If is_drawing is false, we are in a gap (or just finished a dash),
                    // so the pen is already up. We skip this event to avoid "Double End".
                    if self.is_drawing {
                        self.is_drawing = false;
                        return Some(end_event);
                    } else {
                        // Pen is up. We are done with this closed path.
                        // Loop again to fetch the next path from 'iter'.
                        continue;
                    }
                }

                // B. Fetch Next Source Event
                match self.iter.next()? {
                    PathEvent::Begin { at } => {
                        self.current_pos = at;
                        self.target_pos = at;

                        // Reset
                        self.dash_index = 0;
                        self.is_gap = false;
                        self.current_segment_len = self.dash_array.get(0).copied().unwrap_or(0.0);

                        self.is_drawing = true;
                        return Some(PathEvent::Begin { at });
                    }
                    PathEvent::Line { to, .. } => {
                        self.target_pos = to;
                    }
                    PathEvent::End { last, first, close } => {
                        if close {
                            // Valid closed path: set up the closing segment.
                            self.target_pos = first;

                            // CRITICAL FIX: Force 'close: false'.
                            // The dash itself is an open segment that happens to connect to the start.
                            // We must NOT tell Lyon to treat this individual dash as a closed loop.
                            self.closing_event = Some(PathEvent::End {
                                last,
                                first,
                                close: false, // <--- WAS 'close'
                            });
                        } else {
                            // Open path: just reset
                            self.current_pos = first;
                            self.target_pos = first;

                            if self.is_drawing {
                                self.is_drawing = false;
                                // Force close: false here too, just to be safe (though it likely already is)
                                return Some(PathEvent::End {
                                    last,
                                    first,
                                    close: false,
                                });
                            } else {
                                continue;
                            }
                        }
                    }
                    _ => {} // Flattened input only
                }
            }

            // 3. Process Logic
            let vec = self.target_pos - self.current_pos;
            let dist = vec.length();

            if dist < 1e-5 {
                self.current_pos = self.target_pos;
                continue;
            }

            if dist <= self.current_segment_len {
                // Case A: Consume whole segment
                self.current_segment_len -= dist;
                let start = self.current_pos;
                let end = self.target_pos;
                self.current_pos = self.target_pos;

                if !self.is_gap {
                    return Some(PathEvent::Line {
                        from: start,
                        to: end,
                    });
                } else {
                    continue;
                }
            } else {
                // Case B: Split
                let dir = vec / dist;
                let split_point = self.current_pos + dir * self.current_segment_len;
                let start = self.current_pos;

                self.current_pos = split_point;
                let was_gap = self.is_gap;
                self.advance_dash_state();

                if !was_gap {
                    // Dash Finished -> Emit Line -> Queue End
                    self.is_drawing = false;
                    self.pending_event = Some(PathEvent::End {
                        last: split_point,
                        first: Point::new(0.0, 0.0),
                        close: false,
                    });
                    return Some(PathEvent::Line {
                        from: start,
                        to: split_point,
                    });
                } else {
                    // Gap Finished -> Emit Begin
                    self.is_drawing = true;
                    return Some(PathEvent::Begin { at: split_point });
                }
            }
        }
    }
}
