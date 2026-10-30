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
//! string instructions, near control flow and the x87 floating-point unit
//! ([`fpu_op`]), over eight 32-bit registers and the arithmetic flags.  It is
//! **not** an x86 emulator: there are no segments, no protection, no SIMD,
//! no system instructions, and memory is the VM's own flat array.  Anything outside the subset faults with
//! `reva_vm::Fault::BadOpcode` naming the bytes.
//!
//! Instructions are decoded once into [`Insn`] values and cached by address
//! (see reva-core's `machine`).  The decoded form is also the IR that future
//! translators (x86-64, ARM64, WebAssembly) consume; see reva-core's
//! `translate`.
//!
//! This crate is a facade: the parts are their own crates, in the
//! components x86-types (faults, memory, the x87 registers, the
//! instruction set and CPU), x86-decode, x86-alu and x86-exec
//! (docs/design.md 3.13).  Every old path (`reva_x86::mem::Memory`,
//! `reva_x86::Op`, `reva_x86::decode` ...) is re-exported here, with the
//! traits that give `Cpu` its methods ([`Exec`], [`Arith`] ...).
//!
//! Register roles are Reva's, unchanged:
//!
//! | register | role                                     |
//! |----------|------------------------------------------|
//! | EAX      | top of data stack (TOS)                  |
//! | ESI      | data stack pointer; `[ESI]` is NOS       |
//! | ESP      | return stack pointer (in VM memory)      |
//! | EBX ECX EDX EDI EBP | scratch                       |

pub use reva_x86_base::{fault, layout};
pub use reva_x86_decode::table::decode;
pub use reva_x86_exec::control::MiscOps;
pub use reva_x86_exec::data::DataOps;
pub use reva_x86_exec::dispatch::Exec;
pub use reva_x86_fpuexec::dispatch::FpuExec;
pub use reva_x86_fpustate::fpu::Fpu;
pub use reva_x86_mem::memory as mem;
pub use reva_x86_ops::alu::Arith;
pub use reva_x86_ops::muldiv::MulDivs;
pub use reva_x86_ops::operands::Operands;
pub use reva_x86_ops::regs::Regs;
pub use reva_x86_types::cpu::*;
pub use reva_x86_types::fpu_op;
pub use reva_x86_types::op::*;
pub use reva_x86_types::operand::*;
