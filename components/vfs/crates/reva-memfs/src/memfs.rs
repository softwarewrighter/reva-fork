//! The file system: the tree and its open files.

use reva_memtree::tree::MemTree;
use reva_vfs::types::{Access, Errno, errno::EBADF};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

pub use reva_memtree::tree::Change;

/// An open file.
pub struct OpenFile {
    pub path: String,
    pub pos: u64,
    pub access: Access,
}

/// An in-memory file system over the embedded library tree: a
/// [`MemTree`] (whose methods it shares: `put`, `get`, `take_changes`,
/// `set_time` ...) and the files opened in it.
#[derive(Default)]
pub struct MemFs {
    pub tree: MemTree,
    pub open: BTreeMap<u32, OpenFile>,
}

impl MemFs {
    pub fn new() -> MemFs {
        MemFs::default()
    }

    /// The open file `fd`.
    pub fn handle(&mut self, fd: u32) -> Result<&mut OpenFile, Errno> {
        self.open.get_mut(&fd).ok_or(EBADF)
    }
}

impl Deref for MemFs {
    type Target = MemTree;
    fn deref(&self) -> &MemTree {
        &self.tree
    }
}

impl DerefMut for MemFs {
    fn deref_mut(&mut self) -> &mut MemTree {
        &mut self.tree
    }
}
