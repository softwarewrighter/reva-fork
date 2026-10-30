//! `xchg eAX`, `cwde`/`cdq`, string instructions, `test eAX`, `mov imm`.

use reva_x86_base::fault::Fault;
use reva_x86_cursor::cursor::{Cursor, Prefixes};
use reva_x86_types::cpu::EAX;
use reva_x86_types::op::{Op, StrOp};
use reva_x86_types::operand::{Opd, Size};

type R = Result<Op, Fault>;

/// `91`-`97`: `xchg eAX, reg`.
pub fn xchg_eax(_: &mut Cursor, b: u8, p: Prefixes) -> R {
    Ok(Op::Xchg {
        size: p.osz,
        a: Opd::Reg(EAX),
        b: Opd::Reg(b - 0x90),
    })
}

/// `98` `99`: `cwde`/`cdq`, or with `66` `cbw`/`cwd`.
pub fn convert(_: &mut Cursor, b: u8, p: Prefixes) -> R {
    Ok(match (b, p.osz == Size::D) {
        (0x98, true) => Op::Cwde,
        (0x99, true) => Op::Cdq,
        (0x98, false) => Op::Cbw,
        _ => Op::Cwd,
    })
}

/// `A4`-`A7`, `AA`-`AF`: movs, cmps, stos, lods, scas, with `rep`.
pub fn string(_: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b & 1 == 0 { Size::B } else { p.osz };
    let op = match b & !1 {
        0xa4 => StrOp::Movs,
        0xa6 => StrOp::Cmps,
        0xaa => StrOp::Stos,
        0xac => StrOp::Lods,
        _ => StrOp::Scas,
    };
    Ok(Op::Str {
        op,
        size,
        rep: p.rep,
    })
}

/// `A8` `A9`: `test AL/eAX, imm`.
pub fn test_eax(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b == 0xa8 { Size::B } else { p.osz };
    Ok(Op::Test {
        size,
        a: Opd::Reg(EAX),
        b: Opd::Imm(c.imm(size)?),
    })
}

/// `B0`-`BF`: `mov reg, imm`.
pub fn mov_imm(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b < 0xb8 { Size::B } else { p.osz };
    Ok(Op::Mov {
        size,
        dst: Opd::Reg(b & 7),
        src: Opd::Imm(c.imm(size)?),
    })
}
