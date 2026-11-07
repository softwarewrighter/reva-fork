//! # reva-desktop-state — the desktop app's options and state
//!
//! The command line ([`options`]), the state the commands share and the
//! session the page starts from ([`state`]), SQLite in the folder
//! ([`sqlite`]), and what `--about` reports ([`about`]).

pub mod about;
pub mod options;
pub mod sqlite;
pub mod state;
