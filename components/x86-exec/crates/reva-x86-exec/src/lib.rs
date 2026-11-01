//! # reva-x86-exec — executing one vx86 instruction
//!
//! [`dispatch::Exec::exec`] applies a decoded instruction to reva-x86-types'
//! `Cpu` and the VM's memory: it advances EIP and runs the instruction's
//! group ([`data`]: moves, ALU, stack; [`control`]: shifts, conversions,
//! flags, loops), the string instructions (reva-x86-shift) and the x87's
//! (reva-x86-fpuexec).

pub mod control;
pub mod data;
pub mod dispatch;
