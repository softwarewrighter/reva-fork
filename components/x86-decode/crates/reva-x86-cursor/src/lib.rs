//! # reva-x86-cursor — reading an instruction's bytes
//!
//! [`cursor::Cursor`] walks VM memory from an instruction's first byte:
//! immediates, relative branch targets ([`branch`]), ModRM/SIB operands
//! ([`modrm`]), and the fault for bytes the decoder does not know.

pub mod branch;
pub mod cursor;
pub mod modrm;
