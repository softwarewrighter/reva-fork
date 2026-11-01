//! `eval` and `include`: interpret a string or a file, then restore.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::ESI;
use reva_core::{Dispatch, Gate, Machine, Next, st};
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

/// `include FILE`.
pub fn include(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let (a, n) = m.parsews(host)?;
    m.upsh(a)?;
    m.upsh(n)?;
    Ok(Next::Jump(Gate::Include2.addr()))
}

/// `__include` ( a n -- ): slurp, eval, free.  As in the original, the
/// data stack is restored afterwards.
pub fn include2(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    if !m.slurp(host)? {
        m.drop1()?;
        m.drop1()?;
        m.setv(st::IOR, 1)?;
        return Ok(Next::Ret);
    }
    m.rpush(m.nos()?)?;
    m.rpush(m.cpu.r[ESI as usize])?;
    m.rpush(m.eax())?;
    Ok(Next::Call(Gate::Eval.addr(), Gate::IncludeDone))
}

/// After the file: the stack as it was, the text freed.
pub fn include_done(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let eax = m.rpop()?;
    m.cpu.r[ESI as usize] = m.rpop()?;
    m.set_eax(eax);
    m.drop1()?;
    let ptr = m.rpop()?;
    m.heap.free(ptr);
    m.drop1()?;
    m.setv(st::IOR, 0)?;
    Ok(Next::Ret)
}
