//! Decoded instructions.  Branch targets are absolute.

use crate::fpu_op::FpuOp;
use crate::operand::{MemRef, Opd, Size};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alu {
    Add,
    Or,
    Adc,
    Sbb,
    And,
    Sub,
    Xor,
    Cmp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    Rol,
    Ror,
    Rcl,
    Rcr,
    Shl,
    Shr,
    Sar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MulDiv {
    Mul,
    Imul,
    Div,
    Idiv,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrOp {
    Lods,
    Stos,
    Movs,
    Scas,
    Cmps,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rep {
    None,
    /// `rep` / `repe`
    E,
    /// `repne`
    Ne,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopKind {
    Loop,
    Loope,
    Loopne,
    Jecxz,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlagOp {
    Clc,
    Stc,
    Cmc,
    Cld,
    Std,
}

/// A decoded instruction.  Branch targets are absolute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Alu {
        op: Alu,
        size: Size,
        dst: Opd,
        src: Opd,
    },
    Test {
        size: Size,
        a: Opd,
        b: Opd,
    },
    Mov {
        size: Size,
        dst: Opd,
        src: Opd,
    },
    Movx {
        signed: bool,
        from: Size,
        dst: u8,
        src: Opd,
    },
    Lea {
        dst: u8,
        mem: MemRef,
    },
    Xchg {
        size: Size,
        a: Opd,
        b: Opd,
    },
    Inc {
        size: Size,
        dst: Opd,
    },
    Dec {
        size: Size,
        dst: Opd,
    },
    Not {
        size: Size,
        dst: Opd,
    },
    Neg {
        size: Size,
        dst: Opd,
    },
    Push(Opd),
    Pop(Opd),
    Shift {
        op: Shift,
        size: Size,
        dst: Opd,
        count: Opd,
    },
    MulDiv {
        op: MulDiv,
        size: Size,
        src: Opd,
    },
    Imul {
        dst: u8,
        src: Opd,
        imm: Option<u32>,
    },
    Cdq,
    Cwde,
    /// `66 98` cbw: AL sign-extended into AX (the disassembler uses it)
    Cbw,
    /// `66 99` cwd: AX sign-extended into DX:AX
    Cwd,
    /// `bswap r32` (byte order reversal, e.g. for SHA-1's big-endian words)
    Bswap(u8),
    Str {
        op: StrOp,
        size: Size,
        rep: Rep,
    },
    Jcc {
        cc: u8,
        target: u32,
    },
    Setcc {
        cc: u8,
        dst: Opd,
    },
    Cmovcc {
        cc: u8,
        dst: u8,
        src: Opd,
    },
    Jmp(u32),
    JmpInd(Opd),
    Call(u32),
    CallInd(Opd),
    Ret(u16),
    Loop {
        kind: LoopKind,
        target: u32,
    },
    Flag(FlagOp),
    Nop,
    Hlt,
    Int3,
    /// An x87 instruction (`D8`–`DF`); `fwait` decodes as [`Op::Nop`].
    Fpu(FpuOp),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Insn {
    pub op: Op,
    pub len: u8,
}

impl Insn {
    /// True for instructions that end a basic block.
    pub fn ends_block(&self) -> bool {
        matches!(
            self.op,
            Op::Jcc { .. }
                | Op::Jmp(_)
                | Op::JmpInd(_)
                | Op::Call(_)
                | Op::CallInd(_)
                | Op::Ret(_)
                | Op::Loop { .. }
                | Op::Hlt
                | Op::Int3
        )
    }
}
