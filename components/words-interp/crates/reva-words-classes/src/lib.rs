//! # reva-words-classes — word classes, default vectors, `find`
//!
//! Gate handlers (reva-words' dispatch table) for the word classes a
//! dictionary entry's CLASS field points at (called with EAX = xt, EDX =
//! entry), the default targets of DEFER vectors, and dictionary lookup.
//! Each module's `ENTRIES` lists the gates it serves.

pub mod classes;
pub mod data;
pub mod defers;
pub mod lookup;
