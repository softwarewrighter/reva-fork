//! # reva-x86-types — the vx86 instruction set and CPU state
//!
//! Reva compiles Forth to machine code: `compile` emits `call rel32`,
//! `literal` emits `lea esi,[esi-4]; mov [esi],eax; mov eax,imm32`, `if`
//! emits `test eax,eax; lodsd; jz rel32`, and roughly 150 words in `reva.f`
//! and its libraries are written directly as instruction bytes.  The
//! portable core executes the *subset* of 32-bit x86 those bytes use; this
//! crate defines it: operands ([`operand`]), decoded instructions
//! ([`op::Insn`], also the IR future translators consume), the x87's
//! ([`fpu_op`]), and the CPU ([`cpu::Cpu`]).

pub mod cpu;
pub mod fpu_op;
pub mod op;
pub mod operand;
