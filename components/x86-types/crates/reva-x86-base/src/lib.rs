//! # reva-x86-base — faults and the address map of the vx86 VM
//!
//! [`fault::Fault`]: everything that can stop a VM step, with messages
//! that name the cause ([`class::insn_class`] for unsupported code);
//! [`layout`]: the regions of the VM's flat 32-bit memory.

pub mod class;
pub mod display;
pub mod fault;
pub mod layout;
