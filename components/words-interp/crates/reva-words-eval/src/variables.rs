//! `variable` and `variable,`: a header of class `'variable` and a cell.

use crate::define::push_name;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_code::emit::Emit;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Variable => variable;
    Variable2 => variable2;
    Variable2Done => variable2_done;
};

/// `variable NAME`: `variable,` with 0.
pub fn variable(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.upsh(0)?;
    Ok(Next::Jump(Gate::Variable2.addr()))
}

/// `variable,` ( n -- ): the header, with the class `'variable`.
pub fn variable2(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    m.rpush(m.var(st::DEFAULT_CLASS))?;
    m.setv(st::DEFAULT_CLASS, Gate::VClass.addr())?;
    push_name(m, host)?;
    Ok(Next::Call(m.var(st::HEADER_), Gate::Variable2Done))
}

/// After the header: the class back, the cell compiled.
pub fn variable2_done(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let dc = m.rpop()?;
    m.setv(st::DEFAULT_CLASS, dc)?;
    m.comma_n(4)?;
    Ok(Next::Ret)
}
