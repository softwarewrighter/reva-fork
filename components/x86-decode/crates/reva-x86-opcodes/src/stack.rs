//! `inc`/`dec`/`push`/`pop` of a register, `push imm`, `imul imm`, `lea`,
//! `pop r/m`.

use reva_x86_base::fault::Fault;
use reva_x86_cursor::cursor::{Cursor, Prefixes};
use reva_x86_types::op::Op;
use reva_x86_types::operand::{Opd, Size};

type R = Result<Op, Fault>;

/// `40`-`4F`: `inc`/`dec` of a register.
pub fn inc_dec_reg(_: &mut Cursor, b: u8, p: Prefixes) -> R {
    let dst = Opd::Reg(b & 7);
    Ok(if b < 0x48 {
        Op::Inc { size: p.osz, dst }
    } else {
        Op::Dec { size: p.osz, dst }
    })
}

/// `50`-`5F`: `push`/`pop` of a 32-bit register.
pub fn push_pop_reg(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    if p.osz != Size::D {
        return Err(c.bad());
    }
    let r = Opd::Reg(b & 7);
    Ok(if b < 0x58 { Op::Push(r) } else { Op::Pop(r) })
}

/// `68` `6A`: `push imm32` / `push imm8`.
pub fn push_imm(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    if p.osz != Size::D {
        return Err(c.bad());
    }
    let v = if b == 0x68 { c.u32()? } else { c.i8()? };
    Ok(Op::Push(Opd::Imm(v)))
}

/// `69` `6B`: `imul Gv, Ev, imm32/imm8`.
pub fn imul_imm(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    if p.osz != Size::D {
        return Err(c.bad());
    }
    let (r, rm) = c.modrm()?;
    let imm = if b == 0x69 { c.u32()? } else { c.i8()? };
    Ok(Op::Imul {
        dst: r,
        src: rm,
        imm: Some(imm),
    })
}

/// `8D` `lea`, `8F /0` `pop r/m`.
pub fn lea_pop(c: &mut Cursor, b: u8, _: Prefixes) -> R {
    match (b, c.modrm()?) {
        (0x8d, (r, Opd::Mem(m))) => Ok(Op::Lea { dst: r, mem: m }),
        (0x8f, (0, rm)) => Ok(Op::Pop(rm)),
        _ => Err(c.bad()),
    }
}
