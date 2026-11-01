//! What a path is: a file (written or embedded), a directory, its stat.

use crate::tree::MemTree;
use reva_embedded::embedded;
use reva_vfs::types::{Errno, Stat, errno::ENOENT};

impl MemTree {
    /// A file's contents by absolute path.
    pub fn data(&self, p: &str) -> Option<&[u8]> {
        self.files
            .get(p)
            .map(Vec::as_slice)
            .or_else(|| embedded::lib_file(p))
    }

    pub fn is_file(&self, p: &str) -> bool {
        self.data(p).is_some()
    }

    pub fn is_dir(&self, p: &str) -> bool {
        if p == "/" || self.dirs.contains(p) || embedded::lib_dir(p).is_some() {
            return true;
        }
        let prefix = format!("{p}/");
        self.files
            .range(prefix.clone()..)
            .next()
            .is_some_and(|(k, _)| k.starts_with(&prefix))
            || self
                .dirs
                .range(prefix.clone()..)
                .next()
                .is_some_and(|k| k.starts_with(&prefix))
    }

    /// `stat` of an absolute path.
    pub fn stat_abs(&self, p: &str) -> Result<Stat, Errno> {
        let mtime = self.mtimes.get(p).copied().unwrap_or(0);
        if let Some(d) = self.data(p) {
            Ok(Stat {
                size: d.len() as u64,
                dir: false,
                mtime,
            })
        } else if self.is_dir(p) {
            Ok(Stat {
                size: 0,
                dir: true,
                mtime,
            })
        } else {
            Err(ENOENT)
        }
    }
}
