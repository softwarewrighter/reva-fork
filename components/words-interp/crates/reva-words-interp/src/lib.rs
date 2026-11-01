//! # reva-words-interp — startup and the outer interpreter
//!
//! Gate handlers (reva-words' dispatch table) for `cold`, the REPL, and
//! `interpret` with its line editor, split at every call into Forth into
//! a head and continuation gates, so the VM never holds Forth state on
//! the Rust stack and can pause for input anywhere.

pub mod interp;
pub mod outer;
pub mod query;
pub mod startup;
