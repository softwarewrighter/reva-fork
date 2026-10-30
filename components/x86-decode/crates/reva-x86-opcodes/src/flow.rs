//! Short `jcc`, shifts, `ret`, `mov r/m, imm`, `loop`.

use reva_x86_base::fault::Fault;
use reva_x86_cursor::cursor::{Cursor, Prefixes};
use reva_x86_types::cpu::ECX;
use reva_x86_types::op::{LoopKind, Op, Shift};
use reva_x86_types::operand::{Opd, Size};

type R = Result<Op, Fault>;

/// The shifts in `/r` order.
const SHIFT: [Shift; 8] = [
    Shift::Rol,
    Shift::Ror,
    Shift::Rcl,
    Shift::Rcr,
    Shift::Shl,
    Shift::Shr,
    Shift::Shl, // /6 is an undocumented alias of SHL
    Shift::Sar,
];

/// `70`-`7F`: `jcc rel8`.
pub fn jcc8(c: &mut Cursor, b: u8, _: Prefixes) -> R {
    Ok(Op::Jcc {
        cc: b & 15,
        target: c.rel8()?,
    })
}

/// `C0` `C1` `D0`-`D3`: shifts by an immediate, by 1, by CL.
pub fn shift(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b & 1 == 0 { Size::B } else { p.osz };
    let (r, rm) = c.modrm()?;
    let count = match b {
        0xc0 | 0xc1 => Opd::Imm(c.u8()? as u32),
        0xd0 | 0xd1 => Opd::Imm(1),
        _ => Opd::Reg(ECX), // CL
    };
    Ok(Op::Shift {
        op: SHIFT[r as usize],
        size,
        dst: rm,
        count,
    })
}

/// `C2` `ret imm16`, `C3` `ret`.
pub fn ret(c: &mut Cursor, b: u8, _: Prefixes) -> R {
    Ok(Op::Ret(if b == 0xc2 { c.u16()? } else { 0 }))
}

/// `C6 /0` `C7 /0`: `mov r/m, imm`.
pub fn mov_rm_imm(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b == 0xc6 { Size::B } else { p.osz };
    match c.modrm()? {
        (0, rm) => Ok(Op::Mov {
            size,
            dst: rm,
            src: Opd::Imm(c.imm(size)?),
        }),
        _ => Err(c.bad()),
    }
}

/// `E0`-`E3`: loopne, loope, loop, jecxz.
pub fn loop_op(c: &mut Cursor, b: u8, _: Prefixes) -> R {
    let kind = match b {
        0xe0 => LoopKind::Loopne,
        0xe1 => LoopKind::Loope,
        0xe2 => LoopKind::Loop,
        _ => LoopKind::Jecxz,
    };
    Ok(Op::Loop {
        kind,
        target: c.rel8()?,
    })
}
