//! # reva-x86-shift — shifts, rotates, string instructions
//!
//! More of the vx86 executor as methods of reva-x86-types' `Cpu`: the
//! shift group with its flags, and `lods`/`stos`/`movs`/`scas`/`cmps`
//! with `rep`.

pub mod rotate;
pub mod shift;
pub mod string;
