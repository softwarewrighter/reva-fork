//! # reva-memfs — an in-memory file system
//!
//! [`memfs::MemFs`]: reva-memtree's tree with open files, implementing the
//! file-system traits of reva-vfs ([`io`], [`meta`], [`dirs`]).  The
//! browser and tests lend it to the core; it sees the library tree `lib/`
//! at `/reva/lib/` even when nothing is on disk.

pub mod dirs;
pub mod io;
pub mod memfs;
pub mod meta;
