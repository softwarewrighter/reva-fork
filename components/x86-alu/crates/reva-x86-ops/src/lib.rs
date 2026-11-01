//! # reva-x86-ops — operands, the ALU, the stack, multiply and divide
//!
//! The vx86 executor's building blocks as methods of reva-x86-types'
//! `Cpu` (extension traits, `reva_macros::ext_trait!`).  Flags follow the
//! Intel SDM for every instruction in the subset, except AF (not
//! modelled) and the "undefined" flag results, which are left as the
//! nearest sensible value.  Reva code tests ZF, CF, SF and OF (`jz`, `jc`,
//! `sbb eax,eax`, `jle`) and occasionally PF.

pub mod alu;
pub mod muldiv;
pub mod operands;
pub mod regs;
