//! Pop, the status word, condition codes, rounding, reset.

use crate::fpu::{C0, C1, C2, C3, CC, Fpu, TOP_SHIFT};

impl Fpu {
    pub fn pop(&mut self) {
        self.valid &= !(1 << self.phys(0));
        self.top = (self.top + 1) & 7;
    }

    /// The status word as `fnstsw` stores it.
    pub fn status(&self) -> u16 {
        (self.sw & !(7 << TOP_SHIFT)) | (u16::from(self.top) << TOP_SHIFT)
    }

    /// Set the condition codes C3, C2, C1, C0.
    pub fn set_cc(&mut self, c3: bool, c2: bool, c1: bool, c0: bool) {
        self.sw &= !CC;
        for (on, bit) in [(c3, C3), (c2, C2), (c1, C1), (c0, C0)] {
            if on {
                self.sw |= bit;
            }
        }
    }

    /// Round to an integer as the control word's RC field says.
    pub fn round(&self, v: f64) -> f64 {
        match (self.cw >> 10) & 3 {
            0 => v.round_ties_even(),
            1 => v.floor(),
            2 => v.ceil(),
            _ => v.trunc(),
        }
    }

    /// `fninit`.
    pub fn reset(&mut self) {
        *self = Fpu::default();
    }
}
