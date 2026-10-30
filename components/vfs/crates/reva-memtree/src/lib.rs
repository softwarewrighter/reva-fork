//! # reva-memtree — the in-memory file tree
//!
//! [`tree::MemTree`]: files and directories in memory, over the embedded
//! library tree, recording which paths changed (so a page can persist
//! them) and when.  reva-memfs puts open files and the file-system traits
//! on top of it.

pub mod edit;
pub mod lookup;
pub mod public;
pub mod tree;
