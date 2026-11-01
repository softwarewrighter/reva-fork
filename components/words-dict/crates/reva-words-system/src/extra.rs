//! `setextra` and `getextra`: a block of data kept for the program.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next, st};

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    SetExtra => set_extra;
    GetExtra => get_extra;
};

/// `setextra` ( a n -- ).
pub fn set_extra(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let n = m.upop()?;
    let a = m.upop()?;
    m.setv(st::EXTRA_SIZE, n)?;
    m.setv(st::EXTRA_DATA, a)?;
    Ok(Next::Ret)
}

/// `getextra` ( -- a n ).
pub fn get_extra(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.upsh(m.var(st::EXTRA_DATA))?;
    m.upsh(m.var(st::EXTRA_SIZE))?;
    Ok(Next::Ret)
}
