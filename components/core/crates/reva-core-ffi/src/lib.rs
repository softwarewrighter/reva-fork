//! # reva-core-ffi — the arguments of `(call)`, on the machine
//!
//! Helpers the virtual libraries (reva-vlib) and the OS layer
//! (reva-words-os) share: registers by number, `(call)`'s cells read
//! and written in place ([`args::CallArgs`]), and heap buffers kept in
//! statics cells ([`regs::MachineRegs::ffi_buffer`]).

pub mod args;
pub mod regs;
