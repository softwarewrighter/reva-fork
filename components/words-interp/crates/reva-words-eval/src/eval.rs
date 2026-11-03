//! `eval` and `include`: interpret a string or a file, then restore.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::ESI;
use reva_core::{Dispatch, Gate, IncludeFrame, Machine, Next, st};
use reva_core_inspect::source::Source;
use reva_words_calls::heap::HeapWords;
use reva_words_scan::parse::Parse;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Eval => eval;
    EvalDone => eval_done;
    Include => include;
    Include2 => include2;
    IncludeDone => include_done;
    Where => where_;
};

/// `eval` ( a n -- ): interpret the string, keeping the outer source.
pub fn eval(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let n = m.upop()?;
    let a = m.upop()?;
    m.rpush(m.var(st::SOURCE))?;
    m.rpush(m.var(st::TIN))?;
    m.rpush(m.var(st::TP))?;
    m.setv(st::SOURCE, a.wrapping_add(n))?;
    m.setv(st::TIN, a)?;
    Ok(Next::Call(Gate::Interpret.addr(), Gate::EvalDone))
}

/// After `eval`: the outer source again.
pub fn eval_done(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let tp = m.rpop()?;
    let tin = m.rpop()?;
    let src = m.rpop()?;
    m.setv(st::TP, tp)?;
    m.setv(st::TIN, tin)?;
    m.setv(st::SOURCE, src)?;
    Ok(Next::Ret)
}

/// `include FILE`.  Unlike the original, which only set `ioerr`, a file
/// that cannot be read is reported (a diagnostic: stderr on the CLI, the
/// output in the page); `ioerr` is set as before.
pub fn include(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let (a, n) = m.parsews(host)?;
    let name = String::from_utf8_lossy(m.mem.slice(a, n)?).into_owned();
    m.upsh(a)?;
    m.upsh(n)?;
    let next = start_include(m, host)?;
    if matches!(next, Next::Ret) {
        host.diag(&format!("include: {name}: no such file\n"));
    }
    Ok(next)
}

/// `__include` ( a n -- ): slurp, eval, free.  As in the original, the
/// data stack is restored afterwards, and a missing file only sets
/// `ioerr` (`needs` tries several folders this way and reports itself).
pub fn include2(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    start_include(m, host)
}

/// Slurp the file named on the stack and evaluate it (then
/// `include_done`); `Next::Ret` with `ioerr` set if it cannot be read.
pub fn start_include(m: &mut Machine, host: &mut dyn Host) -> R<Next> {
    let (a, n) = (m.nos()?, m.eax());
    let name = String::from_utf8_lossy(m.mem.slice(a, n)?).into_owned();
    if !m.slurp(host)? {
        m.drop1()?;
        m.drop1()?;
        m.setv(st::IOR, 1)?;
        return Ok(Next::Ret);
    }
    // Frames of includes an error abandoned are dropped: the files still
    // being read are the ones whose text holds where this include began.
    let from = m.var(st::TIN);
    let keep = m
        .includes
        .iter()
        .rposition(|f| (f.start..=f.end).contains(&from));
    m.includes.truncate(keep.map_or(0, |k| k + 1));
    let (start, len) = (m.nos()?, m.eax());
    m.includes.push(IncludeFrame {
        name,
        start,
        end: start + len,
        from,
    });
    m.rpush(m.nos()?)?;
    m.rpush(m.cpu.r[ESI as usize])?;
    m.rpush(m.eax())?;
    Ok(Next::Call(Gate::Eval.addr(), Gate::IncludeDone))
}

/// `(where)` ( -- ): print `FILE:LINE: ` (and the including files) while a
/// file is being included; nothing otherwise.  The boot's `ERROR` calls it,
/// so errors in files say where, as a compiler's do.
pub fn where_(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    if let Some(at) = m.source_where() {
        host.write(format!("{at}: ").as_bytes());
    }
    Ok(Next::Ret)
}

/// After the file: the stack as it was, the text freed.
pub fn include_done(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let eax = m.rpop()?;
    m.cpu.r[ESI as usize] = m.rpop()?;
    m.set_eax(eax);
    m.drop1()?;
    let ptr = m.rpop()?;
    if m.includes.last().is_some_and(|f| f.start == ptr) {
        m.includes.pop();
    }
    m.heap.free(ptr);
    m.drop1()?;
    m.setv(st::IOR, 0)?;
    Ok(Next::Ret)
}
