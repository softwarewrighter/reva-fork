//! # reva-ui-term — the TUI in the CLI's terminal
//!
//! The same cells as the web page draws (reva-ui-paint), drawn in the
//! terminal with crossterm (docs/ui-model.md): terminal keys as IUP key
//! codes ([`keys`]), the cells as terminal output, only what changed
//! ([`screen`]), the host's state ([`term`]), and the `ui` host calls it
//! answers ([`host`]).  While a dialog is shown the terminal is in raw
//! mode on its alternate screen; the REPL gets it back before it prompts.

pub mod host;
pub mod keys;
pub mod screen;
pub mod term;
