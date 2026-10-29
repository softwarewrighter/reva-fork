//! `catch` and `throw`: ESP-based frames, exactly as in newcore.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::{EBX, ESI, ESP};
use reva_core::{Dispatch, Gate, Machine, Next, st};

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Catch => catch;
    CatchDone => catch_done;
    Throw => throw;
    NestedReturn => nested_return;
};

/// `catch` ( xt -- 0 | n ): push a frame, run `xt`.
pub fn catch(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.rpush(m.cpu.r[ESI as usize])?;
    m.rpush(m.var(st::HANDLER))?;
    m.setv(st::HANDLER, m.cpu.r[ESP as usize])?;
    let xt = m.upop()?;
    m.cpu.r[EBX as usize] = xt;
    Ok(Next::Call(xt, Gate::CatchDone))
}

/// `xt` returned: drop the frame, leave 0.
pub fn catch_done(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let h = m.rpop()?;
    m.setv(st::HANDLER, h)?;
    m.rpop()?;
    m.upsh(0)?;
    Ok(Next::Ret)
}

/// `throw` ( n -- ): unwind to the innermost frame, unless `n` is 0.
pub fn throw(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let h = m.var(st::HANDLER);
    if m.eax() == 0 || h == 0 {
        m.drop1()?;
        return Ok(Next::Ret);
    }
    m.cpu.r[ESP as usize] = h;
    let prev = m.rpop()?;
    m.setv(st::HANDLER, prev)?;
    m.cpu.r[ESI as usize] = m.rpop()?;
    Ok(Next::Ret)
}

/// The return address of a nested call from Rust into Forth.
pub fn nested_return(m: &mut Machine, g: Gate, _: &mut dyn Host) -> R<Next> {
    if m.nested == 0 {
        return Err(Fault::BadGate { addr: g.addr() });
    }
    Ok(Next::NestedReturn)
}
