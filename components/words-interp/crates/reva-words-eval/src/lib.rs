//! # reva-words-eval — `eval`, `include`, `catch`/`throw`, defining words
//!
//! Gate handlers (reva-words' dispatch table) for the words that run
//! Forth source or Forth code and come back: each is a head gate and a
//! continuation gate, with its state on the VM's return stack.

pub mod catch;
pub mod define;
pub mod eval;
pub mod variables;
