//! The register stack: `st(i)`, push, tags.

use crate::fpu::{C1, Fpu, IE, INDEFINITE, SF};

impl Fpu {
    /// The physical register of `st(i)`.
    pub fn phys(&self, i: u8) -> usize {
        ((self.top + i) & 7) as usize
    }

    pub fn is_empty(&self, i: u8) -> bool {
        self.valid & (1 << self.phys(i)) == 0
    }

    /// `st(i)`; an empty register is a stack underflow: invalid operation,
    /// and the indefinite NaN as the value.
    pub fn st(&mut self, i: u8) -> f64 {
        if self.is_empty(i) {
            self.sw = (self.sw & !C1) | IE | SF;
            return INDEFINITE;
        }
        self.regs[self.phys(i)]
    }

    pub fn set(&mut self, i: u8, v: f64) {
        let p = self.phys(i);
        self.regs[p] = v;
        self.valid |= 1 << p;
    }

    /// Push; onto a full stack this is an overflow (the value becomes the
    /// indefinite NaN), as the x87 does with exceptions masked.
    pub fn push(&mut self, v: f64) {
        self.top = (self.top + 7) & 7;
        if !self.is_empty(0) {
            self.sw |= IE | SF | C1;
            self.set(0, INDEFINITE);
        } else {
            self.set(0, v);
        }
    }
}
