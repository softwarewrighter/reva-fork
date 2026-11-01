//! # reva-words-text — pure text functions of the Reva core's words
//!
//! What `single`, `double`, `(.r)`, `>digit`, `compare` and `fnvhash`
//! compute, as functions of bytes and numbers: no machine, no memory.
//! The words in other crates read their arguments out of VM memory and
//! write the results back.

pub mod compare;
pub mod digits;
pub mod number;
