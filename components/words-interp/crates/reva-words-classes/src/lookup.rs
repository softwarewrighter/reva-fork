//! `find`, the "word?" message, `__header`'s vector, `_hello`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_calls::vectors::Vectors;
use reva_words_code::header::Header;
use reva_words_scan::strings::Strings;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Find => find;
    WordNotFound => word_not_found;
    HeaderDefault => ret |m, h| m.header_default(h);
    Hello => hello;
};

/// `find` ( a n -- entry | a n 0 ).
pub fn find(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let n = m.upop()?;
    let a = m.eax();
    let name = m.mem.slice(a, n)?.to_vec();
    match m.find_entry(&name) {
        Some(e) => m.set_eax(e),
        None => {
            m.upsh(n)?;
            m.upsh(0)?;
        }
    }
    Ok(Next::Ret)
}

/// ( a n -- 0 ): print "name?" and give up on the token.
pub fn word_not_found(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    m.call_type(host)?;
    m.call_emit(host, b'?')?;
    m.call_emit(host, b'\n')?;
    m.upsh(0)?;
    Ok(Next::Ret)
}

/// `_hello`: include argv[1] if given, then run the REPL.
pub fn hello(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    if m.var(st::ARGC) <= 1 {
        return Ok(Next::Jump(Gate::Repl.addr()));
    }
    let p = m.mem.read_u32(m.var(st::ARGV) + 4)?;
    let n = m.zlen(p)?;
    m.upsh(p)?;
    m.upsh(n)?;
    Ok(Next::Call(Gate::Include2.addr(), Gate::Repl))
}
