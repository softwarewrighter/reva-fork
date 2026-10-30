//! # reva-vlib-highlight — syntax highlighting: the virtual library `highlight`
//!
//! `pp` and `hl-type` (`lib/highlight/core`) list Forth source in colour.
//! The words to colour come from token tables, plain text files that the
//! blog's `scripts/forth-highlight.py` reads too, so the REPLs and the
//! blog figures agree (`lib/highlight/forth.tbl`; a package adds its own,
//! such as CoSy's).  A table line is one of
//!
//! ```text
//! style CLASS SGR CSS     how CLASS looks: ANSI SGR codes, then CSS
//! words CLASS WORD ...    these words belong to CLASS
//! | ...                   a comment
//! ```
//!
//! Later lines win, so a later table can recolour a word.  Lexical classes
//! need no words: `com` (`|` to the end of the line), `stk` (`( ... )`),
//! `str` (`"`, `s"`, `."` strings and `'c` characters), `num`, `colon`
//! (`:` `;` `;then`) and `def` (the name after `:`).  A word is looked up
//! as written, then in lower case.
//!
//! Output is the source with each coloured token wrapped in `ESC[SGRm` and
//! `ESC[0m`; the bytes between tokens are copied unchanged.  When the host
//! says colour is off (`Host::color`: the CLI writing to a pipe, say)
//! the source is written exactly as `type` would.  The browser page turns
//! the escapes into spans (`www/ansi.js`).
//!
//! Forth reaches it like `turtle`: `" highlight" lib`, then
//! `hl_type ( src n tables count -- 0 )`, where `tables` holds `count`
//! pairs of cells (address, length) of table texts, and
//! `hl_color ( -- flag )`.

pub mod call;
pub mod lex;
pub mod render;
pub mod table;

pub use call::Highlight;
pub use render::{HL_HANDLE, HL_NAMES, lookup, render};
pub use table::Table;
