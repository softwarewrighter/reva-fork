//! # reva-words-diag — `diagnose`
//!
//! Opt-in checks that stop at the word at fault rather than where the
//! damage shows, and a trace of what is compiled ([`trace`]) (CoSy's ask
//! 22): the same settings in every host, as a Forth word, with the CLI's
//! flags setting them at start.

pub mod diagnose;
pub mod trace;
