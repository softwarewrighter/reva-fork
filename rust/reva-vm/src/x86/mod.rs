//! # vx86 — the IA-32 subset that Reva's code is made of
//!
//! Reva compiles Forth to machine code: `compile` emits `call rel32`,
//! `literal` emits `lea esi,[esi-4]; mov [esi],eax; mov eax,imm32`, `if`
//! emits `test eax,eax; lodsd; jz rel32`, and roughly 150 words in `reva.f`
//! and its libraries are written directly as instruction bytes
//! (`inline{ 01 06 }`, `[ $b8ad063b , ]`).
//!
//! Rather than rewrite all of that, the portable core executes the *subset*
//! of 32-bit x86 those bytes use: integer ALU, moves, shifts, mul/div, the
//! string instructions, and near control flow, over eight 32-bit registers
//! and the arithmetic flags.  It is **not** an x86 emulator: there are no
//! segments, no protection, no FPU (yet), no system instructions, and memory
//! is the VM's own flat array.  Anything outside the subset faults with
//! [`crate::Fault::BadOpcode`] naming the bytes.
//!
//! Instructions are decoded once into [`Insn`] values and cached by address
//! (see [`crate::machine`]).  The decoded form is also the IR that future
//! translators (x86-64, ARM64, WebAssembly) consume; see
//! [`crate::translate`].
//!
//! Register roles are Reva's, unchanged:
//!
//! | register | role                                     |
//! |----------|------------------------------------------|
//! | EAX      | top of data stack (TOS)                  |
//! | ESI      | data stack pointer; `[ESI]` is NOS       |
//! | ESP      | return stack pointer (in VM memory)      |
//! | EBX ECX EDX EDI EBP | scratch                       |

pub mod decode;
pub mod exec;

pub use decode::decode;

pub const EAX: u8 = 0;
pub const ECX: u8 = 1;
pub const EDX: u8 = 2;
pub const EBX: u8 = 3;
pub const ESP: u8 = 4;
pub const EBP: u8 = 5;
pub const ESI: u8 = 6;
pub const EDI: u8 = 7;

pub const REG_NAMES: [&str; 8] = ["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"];

/// Operand size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    B,
    W,
    D,
}

impl Size {
    #[inline]
    pub fn bytes(self) -> u32 {
        match self {
            Size::B => 1,
            Size::W => 2,
            Size::D => 4,
        }
    }
    #[inline]
    pub fn mask(self) -> u32 {
        match self {
            Size::B => 0xff,
            Size::W => 0xffff,
            Size::D => 0xffff_ffff,
        }
    }
    #[inline]
    pub fn sign(self) -> u32 {
        match self {
            Size::B => 0x80,
            Size::W => 0x8000,
            Size::D => 0x8000_0000,
        }
    }
}

/// `[base + index*scale + disp]`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemRef {
    pub base: Option<u8>,
    pub index: Option<u8>,
    pub scale: u8,
    pub disp: u32,
}

/// An operand.  For [`Size::B`] a `Reg(n)` names AL CL DL BL AH CH DH BH.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opd {
    Reg(u8),
    Mem(MemRef),
    Imm(u32),
}

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

/// The CPU state: eight registers, EIP, and the flags Reva code observes.
/// (AF is not modelled; nothing in Reva uses BCD arithmetic.)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cpu {
    pub r: [u32; 8],
    pub eip: u32,
    pub cf: bool,
    pub zf: bool,
    pub sf: bool,
    pub of: bool,
    pub pf: bool,
    pub df: bool,
    /// Lowest legal ESP; pushes below it fault.
    pub esp_floor: u32,
}

impl Cpu {
    /// Evaluate condition code `cc` (the low nibble of `Jcc`/`SETcc`).
    #[inline]
    pub fn cond(&self, cc: u8) -> bool {
        let r = match cc >> 1 {
            0 => self.of,
            1 => self.cf,
            2 => self.zf,
            3 => self.cf || self.zf,
            4 => self.sf,
            5 => self.pf,
            6 => self.sf != self.of,
            _ => self.zf || (self.sf != self.of),
        };
        r ^ (cc & 1 == 1)
    }
}
