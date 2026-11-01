//! Status, removal, renaming, new directories in memory.

use crate::memfs::MemFs;
use reva_embedded::embedded;
use reva_vfs::meta::FsMeta;
use reva_vfs::types::{Errno, Stat, errno::*, parent};

impl FsMeta for MemFs {
    fn fstat(&mut self, fd: u32) -> Result<Stat, Errno> {
        let path = self.handle(fd)?.path.clone();
        self.stat_abs(&path)
    }

    fn stat(&mut self, path: &str) -> Result<Stat, Errno> {
        let p = self.abs(path);
        self.stat_abs(&p)
    }

    fn remove(&mut self, path: &str) -> Result<(), Errno> {
        let p = self.abs(path);
        if self.files.remove(&p).is_some() {
            self.mtimes.remove(&p);
            self.changed.insert(p);
            Ok(())
        } else if embedded::lib_file(&p).is_some() {
            Err(EACCES)
        } else if self.is_dir(&p) {
            Err(EISDIR)
        } else {
            Err(ENOENT)
        }
    }

    /// A file, or a directory with everything below it; embedded paths
    /// are read-only.
    fn rename(&mut self, from: &str, to: &str) -> Result<(), Errno> {
        let (a, b) = (self.abs(from), self.abs(to));
        if !self.is_dir(parent(&b)) {
            return Err(ENOENT);
        }
        if self.files.contains_key(&a) {
            if self.is_dir(&b) {
                return Err(EISDIR);
            }
            self.rename_file(&a, &b);
            return Ok(());
        }
        if embedded::lib_file(&a).is_some() || embedded::lib_dir(&a).is_some() {
            return Err(EACCES);
        }
        if !self.is_dir(&a) {
            return Err(ENOENT);
        }
        if a == "/" || b.starts_with(&format!("{a}/")) {
            return Err(EINVAL);
        }
        self.rename_dir(&a, &b);
        Ok(())
    }

    fn mkdir(&mut self, path: &str) -> Result<(), Errno> {
        let p = self.abs(path);
        if self.is_file(&p) || self.is_dir(&p) {
            return Err(EEXIST);
        }
        if !self.is_dir(parent(&p)) {
            return Err(ENOENT);
        }
        self.dirs.insert(p);
        Ok(())
    }
}
