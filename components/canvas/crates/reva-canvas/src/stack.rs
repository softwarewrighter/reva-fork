//! The turtle-state stack, paths, and what was drawn.

use crate::canvas::{Canvas, MAX_STACK, Segment};

impl Canvas {
    /// Save the turtle's state; false if the stack is full.
    pub fn push(&mut self) -> bool {
        if self.stack.len() >= MAX_STACK {
            return false;
        }
        self.stack.push(self.turtle);
        true
    }

    /// Restore the last saved state (moving without drawing); false if
    /// nothing was saved.
    pub fn pop(&mut self) -> bool {
        match self.stack.pop() {
            Some(t) => {
                self.turtle = t;
                self.moved = true;
                true
            }
            None => false,
        }
    }

    /// Draw a path given as absolute headings: for each, face it and take
    /// one step.  This is the bulk entry point for array languages (CoSy:
    /// headings are a running sum `+\` of turns).  Returns false if the
    /// drawing filled up.
    pub fn path(&mut self, headings: impl IntoIterator<Item = f64>) -> bool {
        for h in headings {
            self.set_heading(h);
            if !self.step() {
                return false;
            }
        }
        true
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// Whether anything was ever drawn or asked for (a host need not open a
    /// window before this).
    pub fn active(&self) -> bool {
        !self.segments.is_empty() || self.cleared || self.moved
    }
}
