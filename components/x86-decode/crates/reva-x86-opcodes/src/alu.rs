//! The ALU opcodes (`00`-`3D`, `80`/`81`/`83`), `test`, `xchg`, `mov`.

use reva_x86_base::fault::Fault;
use reva_x86_cursor::cursor::{Cursor, Prefixes};
use reva_x86_types::cpu::EAX;
use reva_x86_types::op::{Alu, Op};
use reva_x86_types::operand::{Opd, Size};

type R = Result<Op, Fault>;

/// The ALU operations in opcode order (and `/r` order of `80`-`83`).
pub const ALU: [Alu; 8] = [
    Alu::Add,
    Alu::Or,
    Alu::Adc,
    Alu::Sbb,
    Alu::And,
    Alu::Sub,
    Alu::Xor,
    Alu::Cmp,
];

/// `00`-`3D`: add, or, adc, sbb, and, sub, xor, cmp in the forms `Eb,Gb`
/// `Ev,Gv` `Gb,Eb` `Gv,Ev` `AL,Ib` `eAX,Iv`.
pub fn alu_op(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    if b & 7 >= 6 {
        return Err(c.bad());
    }
    let op = ALU[(b >> 3) as usize];
    let size = if b & 1 == 0 { Size::B } else { p.osz };
    let (dst, src) = if b & 7 < 4 {
        let (r, rm) = c.modrm()?;
        if b & 2 == 0 {
            (rm, Opd::Reg(r))
        } else {
            (Opd::Reg(r), rm)
        }
    } else {
        (Opd::Reg(EAX), Opd::Imm(c.imm(size)?))
    };
    Ok(Op::Alu { op, size, dst, src })
}

/// `80` `81` `83`: an ALU operation with an immediate.
pub fn group1(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b == 0x80 { Size::B } else { p.osz };
    let (r, rm) = c.modrm()?;
    let imm = match b {
        0x80 => c.imm(Size::B)?,
        0x81 => c.imm(p.osz)?,
        _ => c.i8()?,
    };
    Ok(Op::Alu {
        op: ALU[r as usize],
        size,
        dst: rm,
        src: Opd::Imm(imm),
    })
}

/// `84` `85`: `test Eb/Ev, Gb/Gv`.
pub fn test_rm(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b == 0x84 { Size::B } else { p.osz };
    let (r, rm) = c.modrm()?;
    Ok(Op::Test {
        size,
        a: rm,
        b: Opd::Reg(r),
    })
}

/// `86` `87`: `xchg Eb/Ev, Gb/Gv`.
pub fn xchg_rm(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b == 0x86 { Size::B } else { p.osz };
    let (r, rm) = c.modrm()?;
    Ok(Op::Xchg {
        size,
        a: rm,
        b: Opd::Reg(r),
    })
}

/// `88`-`8B`: `mov` between a register and r/m.
pub fn mov_rm(c: &mut Cursor, b: u8, p: Prefixes) -> R {
    let size = if b & 1 == 0 { Size::B } else { p.osz };
    let (r, rm) = c.modrm()?;
    let (dst, src) = if b & 2 == 0 {
        (rm, Opd::Reg(r))
    } else {
        (Opd::Reg(r), rm)
    };
    Ok(Op::Mov { size, dst, src })
}
