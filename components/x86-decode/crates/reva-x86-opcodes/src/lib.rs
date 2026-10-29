//! # reva-x86-opcodes — decoders of the one-byte integer opcodes
//!
//! One function per opcode group, each a [`reva_x86_cursor::cursor::Decoder`]
//! that reva-x86-decode's table calls with the opcode byte and the
//! prefixes; the cursor is past the opcode.  A form outside the subset
//! returns the cursor's bad-opcode fault.

pub mod alu;
pub mod flow;
pub mod misc;
pub mod stack;
