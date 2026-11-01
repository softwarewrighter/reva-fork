//! Moving and turning.

use crate::canvas::{Canvas, MAX_SEGMENTS, Segment, sin_cos_deg};

impl Canvas {
    /// Move `dist` pixels along the heading (backwards if negative),
    /// drawing if the pen is down.  Returns false if the drawing is full.
    pub fn forward(&mut self, dist: f64) -> bool {
        let t = self.turtle;
        let (s, c) = sin_cos_deg(t.heading);
        self.move_to(t.x + dist * s, t.y + dist * c)
    }

    /// `forward` by the current step length.
    pub fn step(&mut self) -> bool {
        self.forward(self.turtle.step)
    }

    /// Go to `(x, y)`, drawing if the pen is down.  Returns false if the
    /// drawing is full (the turtle moves anyway).
    pub fn move_to(&mut self, x: f64, y: f64) -> bool {
        let t = self.turtle;
        self.turtle.x = x;
        self.turtle.y = y;
        self.moved = true;
        if !t.pen_down {
            return true;
        }
        if self.segments.len() >= MAX_SEGMENTS {
            return false;
        }
        self.segments.push(Segment {
            x1: t.x as f32,
            y1: t.y as f32,
            x2: x as f32,
            y2: y as f32,
            rgb: t.rgb,
            width: t.width as f32,
        });
        true
    }

    /// Turn clockwise by `deg` degrees (anticlockwise if negative).
    pub fn right(&mut self, deg: f64) {
        self.set_heading(self.turtle.heading + deg);
    }

    pub fn set_heading(&mut self, deg: f64) {
        self.turtle.heading = deg.rem_euclid(360.0);
        self.moved = true;
    }
}
