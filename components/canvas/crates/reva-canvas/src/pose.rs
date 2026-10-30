//! Home, erasing, the step length.

use crate::canvas::{Canvas, Turtle};

impl Canvas {
    /// Back to the centre, facing north (the pen and colour stay).
    pub fn home(&mut self) {
        self.turtle.x = 0.0;
        self.turtle.y = 0.0;
        self.set_heading(0.0);
    }

    /// Erase the drawing (the turtle stays where it is): Logo's `clean`.
    pub fn clean(&mut self) {
        self.segments.clear();
        self.sent = 0;
        self.cleared = true;
        self.moved = true;
        self.full = false;
    }

    /// Erase and start over: drawing, turtle state and stack (Logo's
    /// `clearscreen`).  The step length and visibility are reset too.
    pub fn clear(&mut self) {
        self.clean();
        self.stack.clear();
        self.turtle = Turtle::default();
    }

    /// Multiply the step length by `num/den` (a zero `den` is ignored).
    pub fn scale(&mut self, num: f64, den: f64) {
        if den != 0.0 {
            self.turtle.step *= num / den;
        }
    }
}
