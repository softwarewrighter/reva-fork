//! The dispatch, and the arithmetic and comparison forms.

use crate::load::{d9_mem, d9_reg, da_reg, df_reg};
use crate::memory::{db_mem, db_reg, dd_mem, dd_reg, df_mem};
use reva_x86_base::fault::Fault;
use reva_x86_cursor::cursor::Cursor;
use reva_x86_types::fpu_op::{FArith, FArith::*, FMem, FSrc, FpuOp};
use reva_x86_types::operand::Opd;

/// Arithmetic in `D8 /r` order: dst = st(0) or a memory source.
const D8: [Option<FArith>; 8] = [
    Some(Add),
    Some(Mul),
    None,
    None,
    Some(Sub),
    Some(Subr),
    Some(Div),
    Some(Divr),
];

/// `DC`/`DE` register forms name st(i) as the destination, and swap the
/// plain and reversed subtract and divide.
const DC: [Option<FArith>; 8] = [
    Some(Add),
    Some(Mul),
    None,
    None,
    Some(Subr),
    Some(Sub),
    Some(Divr),
    Some(Div),
];

/// An x87 instruction: `b` is its first byte (`D8`–`DF`).
pub fn fpu_op(c: &mut Cursor, b: u8) -> Result<FpuOp, Fault> {
    let (r, rm) = c.modrm()?;
    let op = match (b, rm) {
        (0xd8 | 0xdc | 0xda | 0xde, Opd::Mem(m)) => Some(d8_form(r, FSrc::Mem(arith_format(b), m))),
        (0xd8, Opd::Reg(i)) => Some(d8_form(r, FSrc::St(i))),
        (0xdc | 0xde, Opd::Reg(i)) => dc_reg(b, r, i),
        (0xd9, Opd::Mem(m)) => d9_mem(r, m),
        (0xd9, Opd::Reg(i)) => d9_reg(r, i),
        (0xda, Opd::Reg(i)) => da_reg(r, i),
        (0xdb, Opd::Mem(m)) => db_mem(r, m),
        (0xdb, Opd::Reg(i)) => db_reg(r, i),
        (0xdd, Opd::Mem(m)) => dd_mem(r, m),
        (0xdd, Opd::Reg(i)) => dd_reg(r, i),
        (0xdf, Opd::Mem(m)) => df_mem(r, m),
        (0xdf, Opd::Reg(i)) => df_reg(r, i),
        _ => None,
    };
    op.ok_or_else(|| c.bad())
}

/// The memory format of `D8`/`DC`/`DA`/`DE` arithmetic.
fn arith_format(b: u8) -> FMem {
    match b {
        0xd8 => FMem::F32,
        0xdc => FMem::F64,
        0xda => FMem::I32,
        _ => FMem::I16,
    }
}

/// `D8 /r` order: arithmetic into st(0), or `fcom`/`fcomp` (/2, /3).
fn d8_form(r: u8, src: FSrc) -> FpuOp {
    match D8[r as usize] {
        Some(op) => FpuOp::Arith {
            op,
            dst: 0,
            src,
            pop: false,
        },
        None => FpuOp::Compare {
            src,
            pops: u8::from(r == 3),
            unordered: false,
        },
    }
}

/// `DC`/`DE` with a register: arithmetic into st(i) (and pop for `DE`),
/// `fcompp`, and `DC D0+i`/`D8+i`, aliases of `fcom`/`fcomp`.
fn dc_reg(b: u8, r: u8, i: u8) -> Option<FpuOp> {
    Some(match (DC[r as usize], b, r, i) {
        (Some(op), ..) => FpuOp::Arith {
            op,
            dst: i,
            src: FSrc::St(0),
            pop: b == 0xde,
        },
        (None, 0xde, 3, 1) => FpuOp::Compare {
            src: FSrc::St(1),
            pops: 2,
            unordered: false,
        },
        (None, 0xdc, _, _) => FpuOp::Compare {
            src: FSrc::St(i),
            pops: u8::from(r == 3),
            unordered: false,
        },
        _ => return None,
    })
}
