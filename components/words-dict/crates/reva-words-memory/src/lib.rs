//! # reva-words-memory — memory, strings, the heap, lists
//!
//! Gate handlers (reva-words' dispatch table) for `2dup`, `move`,
//! `fill`, `place`, `zt`, `zcount`, `compare`, `fnvhash`, `allocate`,
//! `free`, `resize`, `slurp`, `link`, `onexit`, `onstartup`, `iterate`.

pub mod lists;
pub mod memory;
pub mod strings;
