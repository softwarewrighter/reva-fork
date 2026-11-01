//! The file system's state: open files by descriptor.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

use reva_hostpath::paths::{host_path, install_root};
use reva_vm::embedded;
use reva_vm::vfs::{self, Access, Errno, FIRST_FD, FsDirs, errno::*};

/// An open file: on disk, or an embedded library file.
pub enum Handle {
    Disk {
        file: File,
        access: Access,
    },
    /// An embedded library file (read-only).
    Embedded {
        data: &'static [u8],
        pos: u64,
    },
}

#[derive(Default)]
/// The real file system: the process's working directory, with `/reva/`
/// at [`reva_hostpath::paths::install_root`].
pub struct StdFs {
    pub open: BTreeMap<u32, Handle>,
}

impl StdFs {
    /// (virtual absolute path, host path)
    pub fn resolve(&mut self, path: &str) -> (String, PathBuf) {
        let v = vfs::normalize(&self.cwd(), path);
        let host = match embedded::install_relative(&v) {
            Some(rel) => install_root().join(rel),
            None => host_path(&v, cfg!(windows)),
        };
        (v, host)
    }

    pub fn insert(&mut self, h: Handle) -> u32 {
        let fd = (FIRST_FD..)
            .find(|f| !self.open.contains_key(f))
            .unwrap_or(u32::MAX);
        self.open.insert(fd, h);
        fd
    }

    pub fn handle(&mut self, fd: u32) -> Result<&mut Handle, Errno> {
        self.open.get_mut(&fd).ok_or(EBADF)
    }
}
