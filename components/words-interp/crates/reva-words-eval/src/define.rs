//! `header`, `:` and `'`: words that parse a name.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_scan::parse::Parse;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Header => header;
    Colon => colon;
    Tick => tick;
    TickDone => tick_done;
};

/// `header NAME`: through the `(header)` vector.
pub fn header(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    push_name(m, host)?;
    Ok(Next::Jump(m.var(st::HEADER_)))
}

/// `: NAME`: a header, then compile (`]`).
pub fn colon(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    push_name(m, host)?;
    Ok(Next::Call(m.var(st::HEADER_), Gate::RBracket))
}

/// `' NAME`: look it up through `find`'s vector.
pub fn tick(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    push_name(m, host)?;
    Ok(Next::Call(m.var(st::FINDINTERNAL), Gate::TickDone))
}

/// After `find`: the xt, or 0.
pub fn tick_done(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    if m.eax() == 0 {
        m.drop1()?;
        m.drop1()?;
        m.set_eax(0);
    } else {
        let xt = m.mem.read_u32(m.eax() + 4)?;
        m.set_eax(xt);
    }
    Ok(Next::Ret)
}

/// The next word of the input, as ( a n ).
pub fn push_name(m: &mut Machine, host: &mut dyn Host) -> R {
    let (a, n) = m.parsews(host)?;
    m.upsh(a)?;
    m.upsh(n)
}
