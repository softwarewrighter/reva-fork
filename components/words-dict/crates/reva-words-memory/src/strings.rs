//! String and heap words: `zcount`, `compare`, `fnvhash`, `allocate`,
//! `free`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_scan::strings::Strings;
use reva_words_text::compare::fnv_hash;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Zcount => zcount;
    Cmp | Cmpi => compare;
    FnvHash => fnvhash;
    Allocate => allocate;
    Free => free;
};

/// `zcount` ( a -- a n ).
pub fn zcount(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let s = m.eax();
    let n = if s == 0 { 0 } else { m.zlen(s)? };
    m.upsh(n)?;
    Ok(Next::Ret)
}

/// `compare` / `comparei` ( a1 n1 a2 n2 -- r ).
pub fn compare(m: &mut Machine, g: Gate, _: &mut dyn Host) -> R<Next> {
    m.compare(g == Gate::Cmpi)?;
    Ok(Next::Ret)
}

/// `fnvhash` ( a n -- h ).
pub fn fnvhash(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let n = m.upop()?;
    let a = m.eax();
    let h = fnv_hash(m.mem.slice(a, n)?);
    m.set_eax(h);
    Ok(Next::Ret)
}

/// `allocate` ( n -- a ), `ioerr` 1 on failure.
pub fn allocate(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let a = m.heap.alloc(m.eax()).unwrap_or(0);
    m.set_eax(a);
    m.setv(st::IOR, (a == 0) as u32)?;
    Ok(Next::Ret)
}

/// `free` ( a -- ).
pub fn free(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let a = m.upop()?;
    m.heap.free(a);
    m.setv(st::IOR, 0)?;
    Ok(Next::Ret)
}
