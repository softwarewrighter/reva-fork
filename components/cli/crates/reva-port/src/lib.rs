//! # reva-port — from Windows-only Reva programs to portable ones
//!
//! `reva --port-check [--fix DIR] FILE...` (docs/porting-from-windows.md)
//! reports what ties a Forth program to Windows, as `FILE:LINE:COL`, and
//! with `--fix` writes migrated copies to DIR, never over the originals.
//! It does not emulate Windows: hard-coded drives, `\` as the path
//! separator and `del`/`copy` through the shell become portable Forth.
//!
//! [`lex`]: a source line as code, strings and comments; [`rules`]: what
//! is found, and the fix where one is safe; [`migrate`]: a file's fixed
//! copy; [`cli`]: the command line and the report.

pub mod cli;
pub mod lex;
pub mod migrate;
pub mod rules;
