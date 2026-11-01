//! # reva-stdhost — the command-line host
//!
//! [`host::StdHost`]: output to stdout (stderr for the VM's notes), lines
//! from the terminal (reva-term), the process environment and clock, the
//! shell (only with `--allow-shell`), the real file system (reva-stdfs),
//! and SQLite answered at once through host calls (reva-sqlite).

pub mod console;
pub mod host;
pub mod platform;
pub mod resources;
