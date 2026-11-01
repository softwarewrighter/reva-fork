//! # reva-words — the gate dispatch of the Reva core
//!
//! The `newcore.asm` routines are in Rust, in families, each in its own
//! crate (components words-interp, words-dict, words-os; their helpers in
//! words-base).  Every family module lists the gates it serves; this
//! crate assembles the lists into one table, indexed by gate, and
//! [`dispatch::dispatch`] — the function reva-vm builds machines with —
//! runs a gate through it (docs/design.md 3.12, 3.13).

pub mod dispatch;
pub mod table;

pub use reva_words_ffi as ffi;
