//! `reset`, `bye`, `restart`, `version`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::S0;
use reva_core::x86::{ESI, ESP};
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_calls::vectors::Vectors;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Reset => reset;
    Bye => bye;
    Restart => restart;
    Version => version;
};

/// `reset`: empty the data stack.
pub fn reset(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.cpu.r[ESI as usize] = S0;
    m.set_eax(0);
    m.setv(st::RP0_SAVED, m.cpu.r[ESP as usize])?;
    Ok(Next::Ret)
}

/// `bye`: run `onexit` handlers, then exit with TOS as the status.
pub fn bye(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let code = m.eax();
    m.iterate_list(host, Gate::OnStartThunk.addr(), st::EXIT_LIST)?;
    Ok(Next::Halt(code as i32))
}

/// `restart`: like `bye` (onexit handlers run), but the host starts over.
pub fn restart(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    m.iterate_list(host, Gate::OnStartThunk.addr(), st::EXIT_LIST)?;
    Ok(Next::Restart)
}

/// `version`: this build's version report.
pub fn version(_: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    host.write(reva_core::version::info().as_bytes());
    Ok(Next::Ret)
}
