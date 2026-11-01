//! The tree's state.

use reva_vfs::types::normalize;
use std::collections::{BTreeMap, BTreeSet};

/// A change [`MemTree::take_changes`] reports: the file's new contents, or
/// `None` if it was removed.
pub type Change = (String, Option<Vec<u8>>);

/// Files and directories in memory.
///
/// Directories are implicit (any prefix of a file path) or made with
/// `mkdir`; only files are reported as changes, so an empty directory does
/// not outlive the page.  Files under the embedded tree are read-only
/// until written: the first write copies them.
pub struct MemTree {
    pub files: BTreeMap<String, Vec<u8>>,
    pub dirs: BTreeSet<String>,
    pub cwd: String,
    pub changed: BTreeSet<String>,
    /// Seconds since the epoch stamped on writes (see [`MemTree::set_time`]).
    pub time: u64,
    pub mtimes: BTreeMap<String, u64>,
}

impl Default for MemTree {
    fn default() -> MemTree {
        MemTree::new()
    }
}

impl MemTree {
    pub fn new() -> MemTree {
        MemTree {
            files: BTreeMap::new(),
            dirs: BTreeSet::new(),
            cwd: "/".to_string(),
            changed: BTreeSet::new(),
            time: 0,
            mtimes: BTreeMap::new(),
        }
    }

    /// The clock for modification times (the core itself has none).
    pub fn set_time(&mut self, secs: u64) {
        self.time = secs;
    }

    /// `path` made absolute against the working directory.
    pub fn abs(&self, path: &str) -> String {
        normalize(&self.cwd, path)
    }
}
