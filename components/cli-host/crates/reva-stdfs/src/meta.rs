//! Status, removal, renaming, new directories.

use std::fs;

use crate::stdfs::{Handle, StdFs};
use reva_hostpath::paths::{errno, stat_of};
use reva_vm::embedded;
use reva_vm::vfs::{Errno, FsMeta, Stat, errno::*};

impl FsMeta for StdFs {
    fn fstat(&mut self, fd: u32) -> Result<Stat, Errno> {
        match self.handle(fd)? {
            Handle::Disk { file, .. } => file.metadata().map(|m| stat_of(&m)).map_err(errno),
            Handle::Embedded { data, .. } => Ok(Stat {
                size: data.len() as u64,
                ..Stat::default()
            }),
        }
    }

    fn stat(&mut self, path: &str) -> Result<Stat, Errno> {
        let (v, p) = self.resolve(path);
        match fs::metadata(&p) {
            Ok(m) => Ok(stat_of(&m)),
            Err(e) => match (embedded::lib_file(&v), embedded::lib_dir(&v)) {
                (Some(d), _) => Ok(Stat {
                    size: d.len() as u64,
                    ..Stat::default()
                }),
                (None, Some(_)) => Ok(Stat {
                    dir: true,
                    ..Stat::default()
                }),
                _ => Err(errno(e)),
            },
        }
    }

    fn remove(&mut self, path: &str) -> Result<(), Errno> {
        let (_, p) = self.resolve(path);
        if p.is_dir() {
            return Err(EISDIR);
        }
        fs::remove_file(p).map_err(errno)
    }

    fn rename(&mut self, from: &str, to: &str) -> Result<(), Errno> {
        let (_, a) = self.resolve(from);
        let (_, b) = self.resolve(to);
        fs::rename(a, b).map_err(errno)
    }

    fn mkdir(&mut self, path: &str) -> Result<(), Errno> {
        let (_, p) = self.resolve(path);
        fs::create_dir(p).map_err(errno)
    }
}
