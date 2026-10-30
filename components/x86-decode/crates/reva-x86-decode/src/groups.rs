//! `call`/`jmp`, groups 3 (`F6`/`F7`) and 4/5 (`FE`/`FF`), one-byte
//! instructions, `mov` with a 32-bit address.

use reva_x86_base::fault::Fault;
use reva_x86_cursor::cursor::{Cursor, Prefixes};
use reva_x86_types::cpu::EAX;
use reva_x86_types::op::{FlagOp, MulDiv, Op};
use reva_x86_types::operand::{MemRef, Opd, Size};

type R = Result<Op, Fault>;

/// `E8` `call rel32`, `E9` `jmp rel32`, `EB` `jmp rel8`.
pub fn call_jmp(c: &mut Cursor, b: u8, _: Prefixes) -> R {
    Ok(match b {
        0xe8 => Op::Call(c.rel32()?),
        0xe9 => Op::Jmp(c.rel32()?),
        _ => Op::Jmp(c.rel8()?),
    })
}

/// `F6` `F7`: test imm, not, neg, mul, imul, div, idiv.
pub fn group3(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b == 0xf6 { Size::B } else { p.osz };
    let (r, rm) = c.modrm()?;
    Ok(match r {
        0 | 1 => Op::Test {
            size,
            a: rm,
            b: Opd::Imm(c.imm(size)?),
        },
        2 => Op::Not { size, dst: rm },
        3 => Op::Neg { size, dst: rm },
        _ => Op::MulDiv {
            op: [MulDiv::Mul, MulDiv::Imul, MulDiv::Div, MulDiv::Idiv][r as usize - 4],
            size,
            src: rm,
        },
    })
}

/// `FE` `FF`: inc, dec; and for `FF` call, jmp, push through r/m.
pub fn group45(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b == 0xfe { Size::B } else { p.osz };
    Ok(match (b, c.modrm()?) {
        (_, (0, rm)) => Op::Inc { size, dst: rm },
        (_, (1, rm)) => Op::Dec { size, dst: rm },
        (0xff, (2, rm)) => Op::CallInd(rm),
        (0xff, (4, rm)) => Op::JmpInd(rm),
        (0xff, (6, rm)) => Op::Push(rm),
        _ => return Err(c.bad()),
    })
}

/// One-byte instructions: nop, fwait (exceptions are masked, so there is
/// nothing to wait for: a nop), int3, hlt, and the flag operations.
pub fn singles(_: &mut Cursor, b: u8, _: Prefixes) -> R {
    Ok(match b {
        0x90 | 0x9b => Op::Nop,
        0xcc => Op::Int3,
        0xf4 => Op::Hlt,
        0xf5 => Op::Flag(FlagOp::Cmc),
        0xf8 => Op::Flag(FlagOp::Clc),
        0xf9 => Op::Flag(FlagOp::Stc),
        0xfc => Op::Flag(FlagOp::Cld),
        _ => Op::Flag(FlagOp::Std),
    })
}

/// `A0`-`A3`: mov AL/EAX <-> [moffs32] (lib/util/locals emits `A3`).
pub fn mov_moffs(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b & 1 == 0 { Size::B } else { p.osz };
    let m = Opd::Mem(MemRef {
        base: None,
        index: None,
        scale: 1,
        disp: c.u32()?,
    });
    let (dst, src) = if b & 2 == 0 {
        (Opd::Reg(EAX), m)
    } else {
        (m, Opd::Reg(EAX))
    };
    Ok(Op::Mov { size, dst, src })
}
