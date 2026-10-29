//! # reva-words-calls — calling back into Forth, lists, the heap
//!
//! Helpers of the core's words that run Forth code from Rust (DEFER
//! vectors such as `type` and `emit`, `iterate` over a list), build the
//! `onstartup`/`onexit` lists, and move files and blocks into the heap.

pub mod heap;
pub mod lists;
pub mod vectors;
