//! Parsing and number words.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_scan::parse::Parse;
use reva_words_scan::strings::Strings;
use reva_words_text::digits::{digit_value, to_digit};

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Parsews => parsews;
    Parse => ret |m, _h| m.parse();
    ParseEsc => ret |m, _h| m.parse_escaped();
    Single => single;
    Double => double;
    FromDigit => from_digit;
    ToDigit => to_digit_word;
    PrintR => ret |m, _h| m.print_r();
};

/// `parsews` ( -- a n ).
pub fn parsews(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let (a, n) = m.parsews(host)?;
    m.upsh(a)?;
    m.upsh(n)?;
    Ok(Next::Ret)
}

/// `single` ( a n -- n -1 | a n 0 ).
pub fn single(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let (a, n) = (m.nos()?, m.eax());
    match m.single_at(a, n)? {
        Some(v) => {
            m.drop1()?;
            m.set_eax(v);
            m.upsh(u32::MAX)?;
        }
        None => m.upsh(0)?,
    }
    Ok(Next::Ret)
}

/// `double` ( a n -- lo hi -1 | a n 0 ).
pub fn double(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let (a, n) = (m.nos()?, m.eax());
    match m.double_at(a, n)? {
        Some((lo, hi)) => {
            m.set_nos(lo)?;
            m.set_eax(hi);
            m.upsh(u32::MAX)?;
        }
        None => m.upsh(0)?,
    }
    Ok(Next::Ret)
}

/// `digit>` ( c -- n | -1 ) in the current base.
pub fn from_digit(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let v = digit_value(m.eax(), m.var(st::BASE)).unwrap_or(u32::MAX);
    m.set_eax(v);
    Ok(Next::Ret)
}

/// `>digit` ( n -- c ).
pub fn to_digit_word(m: &mut Machine, g: Gate, _: &mut dyn Host) -> R<Next> {
    let c = to_digit(m.eax(), m.var(st::BASE)).ok_or(Fault::DivideError { eip: g.addr() })?;
    m.set_eax(c);
    Ok(Next::Ret)
}
