//! `0F`-escaped opcodes, and the x87's.

use reva_x86_base::fault::Fault;
use reva_x86_cursor::cursor::{Cursor, Prefixes};
use reva_x86_fpudec::forms::fpu_op;
use reva_x86_types::op::Op;
use reva_x86_types::operand::Size;

type R = Result<Op, Fault>;

/// `0F xx`: jcc rel32, setcc, cmovcc, imul, bswap, movzx/movsx.
pub fn escape_0f(c: &mut Cursor, _: u8, p: Prefixes) -> R {
    let b2 = c.u8()?;
    let d = p.osz == Size::D;
    Ok(match b2 {
        0x80..=0x8f => Op::Jcc {
            cc: b2 & 15,
            target: c.rel32()?,
        },
        0x90..=0x9f => Op::Setcc {
            cc: b2 & 15,
            dst: c.modrm()?.1,
        },
        0x40..=0x4f if d => {
            let (r, rm) = c.modrm()?;
            Op::Cmovcc {
                cc: b2 & 15,
                dst: r,
                src: rm,
            }
        }
        0xaf if d => {
            let (r, rm) = c.modrm()?;
            Op::Imul {
                dst: r,
                src: rm,
                imm: None,
            }
        }
        0xc8..=0xcf if d => Op::Bswap(b2 & 7),
        0xb6 | 0xb7 | 0xbe | 0xbf => movx(c, b2)?,
        _ => return Err(c.bad()),
    })
}

/// `0F B6/B7/BE/BF`: movzx/movsx from a byte or a word.
fn movx(c: &mut Cursor, b2: u8) -> R {
    let (r, rm) = c.modrm()?;
    let from = if b2 & 1 == 0 { Size::B } else { Size::W };
    Ok(Op::Movx {
        signed: b2 >= 0xbe,
        from,
        dst: r,
        src: rm,
    })
}

/// `D8`-`DF`: an x87 instruction.
pub fn x87(c: &mut Cursor, b: u8, _: Prefixes) -> R {
    Ok(Op::Fpu(fpu_op(c, b)?))
}
