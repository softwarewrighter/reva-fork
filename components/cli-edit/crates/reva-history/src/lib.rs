//! # reva-history — REPL history
//!
//! REPL history: the lines typed at the prompt, oldest first, kept in a
//! plain text file (one entry per line) so they survive `bye`, crashes and
//! the `run-cosy` restart loop.
//!
//! Where: `$REVA_HISTORY` if set (empty: keep history in memory only),
//! else `~/.reva_history`.  A package's launcher picks its own file
//! (`run-cosy` uses `~/.cosy_history`), so packages keep their histories
//! apart, as the web page does with its per-package storage key.
//!
//! Each accepted line is appended to the file at once (no rewrite, so two
//! sessions at a time lose nothing; their lines interleave).  The file is
//! rewritten only when loading finds it at more than twice [`MAX`]
//! entries, to keep it bounded.  Blank lines and a repeat of the last entry
//! are not recorded.  Errors are ignored: history is a convenience and must
//! never stop the REPL.

pub mod entries;
pub mod history;
