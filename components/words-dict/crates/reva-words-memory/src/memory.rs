//! Memory words: `2dup`, `move`, `fill`, `place`, `+place`, `zt`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::ESI;
use reva_core::{Dispatch, Gate, Machine, Next};
use reva_words_calls::heap::HeapWords;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    TwoDup => two_dup;
    Move => move_bytes;
    Fill => fill;
    Place | PlusPlace => place;
    Zt => zt;
    Resize => ret |m, _h| m.resize();
    Slurp => ret |m, h| m.slurp(h);
};

/// `2dup`.
pub fn two_dup(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let a = m.nos()?;
    let b = m.eax();
    let sp = m.cpu.r[ESI as usize].wrapping_sub(8);
    m.mem.write_u32(sp, a)?;
    m.mem.write_u32(sp + 4, b)?;
    m.cpu.r[ESI as usize] = sp;
    Ok(Next::Ret)
}

/// `move` ( src dst n -- ).
pub fn move_bytes(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let n = m.upop()?;
    let dst = m.upop()?;
    let src = m.upop()?;
    m.mem.move_forward(src, dst, n)?;
    Ok(Next::Ret)
}

/// `fill` ( a n c -- ).
pub fn fill(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let c = m.upop()?;
    let n = m.upop()?;
    let a = m.upop()?;
    m.mem.fill(a, n, c as u8)?;
    Ok(Next::Ret)
}

/// `place` / `+place` ( a n buf -- ): a counted, zero-terminated string.
pub fn place(m: &mut Machine, g: Gate, _: &mut dyn Host) -> R<Next> {
    let buf = m.upop()?;
    let len = m.upop()?;
    let s = m.upop()?;
    if buf == 0 {
        return Ok(Next::Ret);
    }
    let dest = if g == Gate::Place {
        if s == 0 {
            m.mem.write_u8(buf, 0)?;
            return Ok(Next::Ret);
        }
        m.mem.write_u8(buf, len as u8)?;
        buf + 1
    } else {
        let cnt = m.mem.read_u8(buf)?;
        m.mem.write_u8(buf, cnt.wrapping_add(len as u8))?;
        buf + cnt as u32 + 1
    };
    m.mem.move_forward(s, dest, len)?;
    m.mem.write_u8(dest + len, 0)?;
    Ok(Next::Ret)
}

/// `zt` ( a n -- a ): zero-terminate in place.
pub fn zt(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let len = m.upop()?;
    let s = m.eax();
    if s != 0 {
        m.mem.write_u8(s.wrapping_add(len), 0)?;
    }
    Ok(Next::Ret)
}
