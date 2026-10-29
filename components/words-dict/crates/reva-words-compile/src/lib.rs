//! # reva-words-compile — compiling words, parsing and number words
//!
//! Gate handlers (reva-words' dispatch table) for `literal`, `;`, `;;`,
//! `[`, `]`, `compile`, `,` and friends, and for `parsews`, `parse`,
//! `single`, `double`, `>digit`, `digit>`, `(.r)`.

pub mod compiling;
pub mod numbers;
