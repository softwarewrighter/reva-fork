//! # reva-words-code — compiling into code space
//!
//! The newcore routines that lay down x86 code and dictionary entries:
//! `compile`, `literal`, `,` and friends, `_align`, `here,`, `;;`,
//! `;inline`, `[` and `__header`, as methods of reva-core's `Machine`.

pub mod emit;
pub mod header;
pub mod semi;
pub mod space;
