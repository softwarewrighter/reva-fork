//! # reva-x86-fpudec — decoding x87 instructions
//!
//! `D8`–`DF` and their ModRM byte to an [`reva_x86_types::fpu_op::FpuOp`]:
//! [`forms::fpu_op`] dispatches on the opcode and the operand kind; each
//! form is a pure function of the `reg` field and the operand, `None` for
//! an encoding outside the subset (`fldenv`, `fsave`, `fbld` ...).

pub mod forms;
pub mod load;
pub mod memory;
