//! What an embedder does with the tree: preload, read, collect changes.

use crate::tree::{Change, MemTree};
use reva_vfs::types::normalize;

impl MemTree {
    /// Add or replace a file without reporting it as a change (preloading
    /// bundled files, or restoring saved ones).
    pub fn put(&mut self, path: &str, data: &[u8]) {
        let p = normalize(&self.cwd, path);
        self.files.insert(p, data.to_vec());
    }

    /// A file's contents (written or embedded).
    pub fn get(&self, path: &str) -> Option<&[u8]> {
        let p = normalize(&self.cwd, path);
        self.data(&p)
    }

    /// Every file written or removed since the last call, by path.
    pub fn take_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changed)
            .into_iter()
            .map(|p| {
                let data = self.files.get(&p).cloned();
                (p, data)
            })
            .collect()
    }

    /// Whether [`MemTree::take_changes`] has anything to report.
    pub fn has_changes(&self) -> bool {
        !self.changed.is_empty()
    }
}
