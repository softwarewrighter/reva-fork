//! The virtual file system (reva-vfs, reva-memfs), under its old path.

pub use reva_memfs::memfs::{Change, MemFs};
pub use reva_vfs::dirs::{FileSystem, FsDirs};
pub use reva_vfs::io::FsFiles;
pub use reva_vfs::meta::FsMeta;
pub use reva_vfs::types::*;
