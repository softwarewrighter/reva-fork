//! Directories in memory.

use crate::memfs::MemFs;
use reva_embedded::embedded;
use reva_vfs::dirs::FsDirs;
use reva_vfs::types::{Errno, errno::*};
use std::collections::BTreeSet;

impl FsDirs for MemFs {
    fn rmdir(&mut self, path: &str) -> Result<(), Errno> {
        let p = self.abs(path);
        if p == "/" {
            return Err(EBUSY);
        }
        if self.is_file(&p) {
            return Err(ENOTDIR);
        }
        if !self.is_dir(&p) {
            return Err(ENOENT);
        }
        if !self.list_dir(&p)?.is_empty() {
            return Err(ENOTEMPTY);
        }
        self.dirs.remove(&p);
        Ok(())
    }

    fn list_dir(&mut self, path: &str) -> Result<Vec<String>, Errno> {
        let p = self.abs(path);
        if self.is_file(&p) {
            return Err(ENOTDIR);
        }
        if !self.is_dir(&p) {
            return Err(ENOENT);
        }
        let prefix = if p == "/" { p.clone() } else { format!("{p}/") };
        let mut names: BTreeSet<String> = BTreeSet::new();
        let child = |k: &str| {
            k.strip_prefix(&prefix)
                .filter(|r| !r.is_empty())
                .map(|r| r.split('/').next().unwrap_or(r).to_string())
        };
        names.extend(self.files.keys().filter_map(|k| child(k)));
        names.extend(self.dirs.iter().filter_map(|k| child(k)));
        names.extend(embedded::lib_dir(&p).unwrap_or_default());
        Ok(names.into_iter().collect())
    }

    fn chdir(&mut self, path: &str) -> Result<(), Errno> {
        let p = self.abs(path);
        if self.is_file(&p) {
            return Err(ENOTDIR);
        }
        if !self.is_dir(&p) {
            return Err(ENOENT);
        }
        self.cwd = p;
        Ok(())
    }

    fn cwd(&mut self) -> String {
        self.cwd.clone()
    }
}
