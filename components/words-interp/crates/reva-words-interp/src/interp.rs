//! `interpret`: read a line (`query`), then each word of it.

use crate::outer::Outer;
use crate::query::Query;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::{EOF_KEY, S0, TIB};
use reva_core::x86::ESI;
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_calls::vectors::Vectors;
use reva_words_scan::parse::Parse;
use reva_words_scan::strings::Strings;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Interpret => interpret;
    QueryKey => query_key;
    QueryKeyGot => query_key_got;
    InterpWord => interp_word;
    InterpFound => interp_found;
};

/// `interpret`: the rest of an `eval` string, or a new line from the
/// keyboard after the prompt.
pub fn interpret(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let src = m.var(st::SOURCE);
    if src != 0 {
        if m.var(st::TIN) >= src {
            return Ok(Next::Ret);
        }
        m.setv(st::TP, src)?;
        return Ok(Next::Jump(Gate::InterpWord.addr()));
    }
    m.call_vector(host, st::PROMPT)?;
    m.setv(st::TP, TIB)?;
    m.setv(st::TIN, TIB)?;
    Ok(Next::Jump(Gate::QueryKey.addr()))
}

/// `query`'s keys: from the input queue, or through a redefined `key`.
pub fn query_key(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let key = m.var(st::KEY);
    if key != Gate::OsKey.addr() {
        return Ok(Next::Call(key, Gate::QueryKeyGot));
    }
    loop {
        let c = match m.input.pop_front() {
            Some(b) => b as u32,
            None if m.input_closed => EOF_KEY,
            None => return Ok(Next::NeedInput),
        };
        if let Some(next) = m.query_char(c)? {
            return Ok(next);
        }
    }
}

/// A key from a redefined `key`.
pub fn query_key_got(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let c = m.upop()?;
    Ok(m.query_char(c)?
        .unwrap_or(Next::Jump(Gate::QueryKey.addr())))
}

/// The next word of the line, looked up through `find`'s vector.
pub fn interp_word(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    if m.cpu.r[ESI as usize] > S0 {
        return Err(Fault::StackUnderflow);
    }
    let (a, n) = m.parsews(host)?;
    if n == 0 {
        return Ok(Next::Jump(Gate::Interpret.addr()));
    }
    m.upsh(a)?;
    m.upsh(n)?;
    Ok(Next::Call(m.var(st::FINDINTERNAL), Gate::InterpFound))
}

/// After `find`: a word, else a single, a double, or ask `isword?`.
pub fn interp_found(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    if m.eax() != 0 {
        return m.after_ok(1);
    }
    m.drop1()?;
    let (a, n) = (m.nos()?, m.eax());
    if let Some(v) = m.single_at(a, n)? {
        m.drop1()?;
        m.set_eax(v);
        return m.after_ok(2);
    }
    if let Some((lo, hi)) = m.double_at(a, n)? {
        m.set_nos(lo)?;
        m.set_eax(hi);
        return m.after_ok(3);
    }
    Ok(Next::Call(m.var(st::IS_WORD), Gate::InterpIsWord))
}
