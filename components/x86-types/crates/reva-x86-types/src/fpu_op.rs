//! x87 instructions, decoded.

use crate::operand::MemRef;

/// A memory operand's format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FMem {
    F32,
    F64,
    F80,
    I16,
    I32,
    I64,
}

/// `dst = dst op src`; the `r` forms reverse the operands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FArith {
    Add,
    Mul,
    Sub,
    Subr,
    Div,
    Divr,
}

/// A source: a memory operand or `st(i)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FSrc {
    St(u8),
    Mem(FMem, MemRef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FConst {
    One,
    L2t,
    L2e,
    Pi,
    Lg2,
    Ln2,
    Zero,
}

/// Operations on `st(0)` in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FUnary {
    Chs,
    Abs,
    Sqrt,
    Rndint,
    F2xm1,
    Sin,
    Cos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FpuOp {
    /// `fld`/`fild` from memory, `fld st(i)`, `fld1` ...
    Load(FSrc),
    LoadConst(FConst),
    /// `fst`/`fstp`/`fist`/`fistp`/`fisttp` (`trunc`) to memory
    Store {
        to: FMem,
        mem: MemRef,
        pop: bool,
        trunc: bool,
    },
    /// `fst st(i)` / `fstp st(i)`
    StoreSt {
        i: u8,
        pop: bool,
    },
    /// `st(dst) = st(dst) op src`, then pop if asked
    Arith {
        op: FArith,
        dst: u8,
        src: FSrc,
        pop: bool,
    },
    /// `fcom`/`fucom` (and `p`, `pp` forms): `st(0)` against `src`
    Compare {
        src: FSrc,
        pops: u8,
        unordered: bool,
    },
    /// `fcomi`/`fucomi` (and `p`): the result goes to EFLAGS
    Comi {
        i: u8,
        pop: bool,
        unordered: bool,
    },
    /// `fcmovcc st(0), st(i)`; `cc` is the integer condition code
    Cmov {
        cc: u8,
        i: u8,
    },
    Tst,
    Xam,
    Xch(u8),
    Free(u8),
    Unary(FUnary),
    Yl2x,
    Yl2xp1,
    Ptan,
    Patan,
    Xtract,
    Prem,
    Prem1,
    Scale,
    Sincos,
    Decstp,
    Incstp,
    Ldcw(MemRef),
    Stcw(MemRef),
    /// `fnstsw m16`, or `fnstsw ax` (`None`)
    Stsw(Option<MemRef>),
    Init,
    Clex,
    Nop,
}
