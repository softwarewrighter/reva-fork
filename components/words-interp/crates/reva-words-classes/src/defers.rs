//! Default targets of DEFER vectors: `noop`, the console, `bye`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::EOF_KEY;
use reva_core::{Dispatch, Gate, Machine, Next};

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Noop => noop;
    OsEmit => os_emit;
    OsType => os_type;
    OsKey => os_key;
    OsBye => os_bye;
};

/// `noop`.
pub fn noop(_: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    Ok(Next::Ret)
}

/// `emit` ( c -- ) to the host.
pub fn os_emit(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let c = m.upop()?;
    host.write(&[c as u8]);
    Ok(Next::Ret)
}

/// `type` ( a n -- ).  The original passed `n` to write(2); a negative
/// count (as `srcstr` yields at the keyboard: `src` is 0 there) made the
/// kernel refuse with EFAULT and print nothing.  Same here.
pub fn os_type(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let n = m.upop()?;
    let a = m.upop()?;
    if n != 0 && (n as i32) > 0 {
        host.write(m.mem.slice(a, n)?);
    }
    Ok(Next::Ret)
}

/// `key` ( -- c ): the next input byte, or wait for input.
pub fn os_key(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    match m.input.pop_front() {
        Some(b) => m.upsh(b as u32)?,
        None if m.input_closed => m.upsh(EOF_KEY)?,
        None => return Ok(Next::NeedInput),
    }
    Ok(Next::Ret)
}

/// `bye` without the `onexit` handlers: halt with TOS as the status.
pub fn os_bye(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    Ok(Next::Halt(m.eax() as i32))
}
