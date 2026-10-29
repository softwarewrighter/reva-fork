//! `cold`, the restart at the end of the boot, the REPL loop.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::{RP0, S0, TIB};
use reva_core::x86::{ESI, ESP};
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_calls::vectors::Vectors;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Cold => cold;
    BootRestart => boot_restart;
    ColdDone => cold_done;
    Repl => repl;
    OnStartThunk => on_start_thunk;
};

/// `cold`: fresh stacks, the `onstartup` list, then `appstart`.
pub fn cold(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    m.cpu.r[ESI as usize] = S0;
    m.set_eax(0);
    m.setv(st::RP0_SAVED, m.cpu.r[ESP as usize])?;
    m.iterate_list(host, Gate::OnStartThunk.addr(), st::START_LIST)?;
    Ok(Next::Call(m.var(st::APPSTART), Gate::ColdDone))
}

/// reva.f's last step saved a native executable; running that executable
/// started with fresh stacks and no source file, ran the `onstartup` list
/// (which allocates `scratch`, `pad` ...) and then `appstart`.  Do the
/// same in place: drop the boot file from argv, abandon the include
/// frames, restart.  TOS is reva.f's interpreter loop, for fault recovery.
pub fn boot_restart(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let repl = m.upop()?;
    m.setv(st::REPL_ENTRY, repl)?;
    m.setv(st::ARGC, 1)?;
    m.setv(st::SOURCE, 0)?;
    m.setv(st::TP, TIB)?;
    m.setv(st::TIN, TIB)?;
    m.setv(st::IS_COMPILING, 0)?;
    m.cpu.r[ESP as usize] = RP0;
    Ok(Next::Jump(Gate::Cold.addr()))
}

/// `appstart` returned: `bye` with status -1.
pub fn cold_done(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.set_eax(u32::MAX);
    Ok(Next::Jump(Gate::Bye.addr()))
}

/// The REPL: `interpret` forever.
pub fn repl(_: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    Ok(Next::Call(Gate::Interpret.addr(), Gate::Repl))
}

/// Run the xt stored in a list node (for `iterate` over handler lists).
pub fn on_start_thunk(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    Ok(Next::Jump(m.mem.read_u32(m.eax())?))
}
