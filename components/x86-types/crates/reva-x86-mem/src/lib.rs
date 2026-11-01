//! # reva-x86-mem — the VM's memory
//!
//! [`memory::Memory`]: one bounds-checked byte array addressed by `u32`,
//! with a null guard and code-page tracking for self-modifying code.

pub mod access;
pub mod bulk;
pub mod memory;
pub mod pages;
