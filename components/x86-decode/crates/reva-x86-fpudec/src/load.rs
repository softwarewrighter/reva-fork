//! `D9` (loads, constants, transcendentals), `DA` and `DF` registers,
//! and the store form.

use reva_x86_types::fpu_op::{FConst, FMem, FSrc, FUnary, FpuOp};
use reva_x86_types::operand::MemRef;

/// `D9 E8+i`: fld1, fldl2t, fldl2e, fldpi, fldlg2, fldln2, fldz.
const D9_CONST: [FConst; 7] = [
    FConst::One,
    FConst::L2t,
    FConst::L2e,
    FConst::Pi,
    FConst::Lg2,
    FConst::Ln2,
    FConst::Zero,
];

/// `D9 F0+i`.
const D9_F0: [FpuOp; 8] = [
    FpuOp::Unary(FUnary::F2xm1),
    FpuOp::Yl2x,
    FpuOp::Ptan,
    FpuOp::Patan,
    FpuOp::Xtract,
    FpuOp::Prem1,
    FpuOp::Decstp,
    FpuOp::Incstp,
];

/// `D9 F8+i`.
const D9_F8: [FpuOp; 8] = [
    FpuOp::Prem,
    FpuOp::Yl2xp1,
    FpuOp::Unary(FUnary::Sqrt),
    FpuOp::Sincos,
    FpuOp::Unary(FUnary::Rndint),
    FpuOp::Scale,
    FpuOp::Unary(FUnary::Sin),
    FpuOp::Unary(FUnary::Cos),
];

/// `D9` with memory: fld m32, fst/fstp m32, fldcw, fnstcw.
pub fn d9_mem(r: u8, m: MemRef) -> Option<FpuOp> {
    Some(match r {
        0 => FpuOp::Load(FSrc::Mem(FMem::F32, m)),
        2 | 3 => store(FMem::F32, m, r == 3, false),
        5 => FpuOp::Ldcw(m),
        7 => FpuOp::Stcw(m),
        _ => return None,
    })
}

/// `D9` with a register: fld st(i), fxch, fnop, fchs, fabs, ftst, fxam,
/// the constants, and the `F0`/`F8` rows.
pub fn d9_reg(r: u8, i: u8) -> Option<FpuOp> {
    Some(match (r, i) {
        (0, _) => FpuOp::Load(FSrc::St(i)),
        (1, _) => FpuOp::Xch(i),
        (2, 0) => FpuOp::Nop,
        (4, 0) => FpuOp::Unary(FUnary::Chs),
        (4, 1) => FpuOp::Unary(FUnary::Abs),
        (4, 4) => FpuOp::Tst,
        (4, 5) => FpuOp::Xam,
        (5, 0..=6) => FpuOp::LoadConst(D9_CONST[i as usize]),
        (6, _) => D9_F0[i as usize],
        (7, _) => D9_F8[i as usize],
        _ => return None,
    })
}

/// `DA` with a register: fcmovb, fcmove, fcmovbe, fcmovu, fucompp.
pub fn da_reg(r: u8, i: u8) -> Option<FpuOp> {
    Some(match r {
        0..=3 => FpuOp::Cmov {
            cc: [0x2, 0x4, 0x6, 0xa][r as usize],
            i,
        },
        5 if i == 1 => FpuOp::Compare {
            src: FSrc::St(1),
            pops: 2,
            unordered: true,
        },
        _ => return None,
    })
}

/// `DF` with a register: fnstsw ax, fucomip, fcomip.
pub fn df_reg(r: u8, i: u8) -> Option<FpuOp> {
    Some(match (r, i) {
        (4, 0) => FpuOp::Stsw(None),
        (5 | 6, _) => FpuOp::Comi {
            i,
            pop: true,
            unordered: r == 5,
        },
        _ => return None,
    })
}

/// A store of st(0) to memory.
pub fn store(to: FMem, mem: MemRef, pop: bool, trunc: bool) -> FpuOp {
    FpuOp::Store {
        to,
        mem,
        pop,
        trunc,
    }
}
