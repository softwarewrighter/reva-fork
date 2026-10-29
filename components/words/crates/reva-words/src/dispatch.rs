//! The function reva-vm builds machines with: every gateway runs through it.

use crate::table::TABLE;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Gate, Machine, Next};

/// The machine's dispatch: run gateway `g` through the table.
pub fn dispatch(m: &mut Machine, g: Gate, host: &mut dyn Host) -> Result<Next, Fault> {
    TABLE[g as usize](m, g, host)
}
