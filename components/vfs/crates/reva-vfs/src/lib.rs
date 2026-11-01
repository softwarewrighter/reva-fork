//! # reva-vfs — the file system a host lends the core
//!
//! Types and `errno` values ([`types`]) and the operations behind `reva.f`'s
//! file words, as three traits a file system implements ([`io::FsFiles`],
//! [`meta::FsMeta`], [`dirs::FsDirs`]) and their union
//! [`dirs::FileSystem`], which a host returns from `Host::fs`.

pub mod dirs;
pub mod io;
pub mod meta;
pub mod types;
