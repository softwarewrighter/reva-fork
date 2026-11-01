//! # reva-x86-fpustate — the x87's registers
//!
//! The state of the floating-point unit: eight registers as a stack with
//! tags and `TOP`, the control word (rounding mode) and the status word
//! (condition codes `C0`–`C3`, sticky exception flags), with every
//! exception masked, as Reva left them.  Registers hold `f64`; the
//! executor (reva-x86-fpuexec) documents the precision this gives.

pub mod control;
pub mod fpu;
pub mod stack;
