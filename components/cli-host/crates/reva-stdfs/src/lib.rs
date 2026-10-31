//! # reva-stdfs — the real file system, for `reva.f`'s file words on the command line.
//!
//! Paths are resolved against the process's working directory (which
//! `chdir` changes).  The core's virtual install prefix `/reva/` maps to
//! reva-hostpath's `install_root`; below it, `lib/` falls back to the copy compiled into
//! the core when the file is not on disk, so an installed `reva` binary
//! runs `needs` without a checkout.

pub mod dirs;
pub mod files;
pub mod meta;
pub mod stdfs;
