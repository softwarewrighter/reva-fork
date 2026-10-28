//! The function reva-vm builds machines with: every gateway runs through it.

use crate::fault::Fault;
use crate::gates::Words;
use crate::host::Host;
use crate::{Gate, Machine, Next};

/// The machine's dispatch: run gateway `g`.
pub fn dispatch(m: &mut Machine, g: Gate, host: &mut dyn Host) -> Result<Next, Fault> {
    m.gate(g, host)
}
