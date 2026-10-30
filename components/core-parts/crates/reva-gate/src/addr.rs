//! Gates and their addresses.

use crate::gate::Gate;
use reva_x86_base::layout::{GATE_BASE, GATE_STRIDE};

impl Gate {
    /// The gate's slot in the gateway region.
    #[inline]
    pub fn addr(self) -> u32 {
        GATE_BASE + GATE_STRIDE * self as u32
    }

    /// The gate whose slot is at `a`.
    #[inline]
    pub fn from_addr(a: u32) -> Option<Gate> {
        let off = a.checked_sub(GATE_BASE)?;
        if off % GATE_STRIDE != 0 {
            return None;
        }
        Gate::ALL.get((off / GATE_STRIDE) as usize).copied()
    }
}
