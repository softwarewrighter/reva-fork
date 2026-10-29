//! # reva-x86-decode — the vx86 decoder
//!
//! Bytes in VM memory → [`reva_x86_types::op::Insn`].  Covers the one-byte
//! and `0F`-escaped opcodes that appear in `newcore.asm` output, `reva.f`,
//! and the integer parts of the libraries, plus their obvious neighbours
//! (all eight ALU ops in every form, all shift kinds, all condition
//! codes), and the x87.  The `66` operand-size prefix is honoured for the
//! generic `Ev`/`Gv`/`Iv` forms; `F2`/`F3` for string instructions.
//!
//! [`table::decode`] reads the prefixes and calls the decoder its table
//! has for the opcode byte (reva-x86-opcodes, [`groups`], [`escape`]).

pub mod escape;
pub mod groups;
pub mod table;
