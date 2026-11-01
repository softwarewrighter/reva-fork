//! Directories and the working directory.

use std::fs;

use crate::stdfs::StdFs;
use reva_hostpath::paths::{errno, vm_path};
use reva_vm::embedded;
use reva_vm::vfs::{Errno, FsDirs};

impl FsDirs for StdFs {
    fn rmdir(&mut self, path: &str) -> Result<(), Errno> {
        let (_, p) = self.resolve(path);
        fs::remove_dir(p).map_err(errno)
    }

    fn list_dir(&mut self, path: &str) -> Result<Vec<String>, Errno> {
        let (v, p) = self.resolve(path);
        let mut names: Vec<String> = match fs::read_dir(&p) {
            Ok(rd) => rd
                .flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect(),
            Err(e) => match embedded::lib_dir(&v) {
                Some(_) => Vec::new(),
                None => return Err(errno(e)),
            },
        };
        for n in embedded::lib_dir(&v).unwrap_or_default() {
            if !names.contains(&n) {
                names.push(n);
            }
        }
        Ok(names)
    }

    fn chdir(&mut self, path: &str) -> Result<(), Errno> {
        let (_, p) = self.resolve(path);
        std::env::set_current_dir(p).map_err(errno)
    }

    fn cwd(&mut self) -> String {
        std::env::current_dir()
            .map(|p| vm_path(&p.to_string_lossy(), cfg!(windows)))
            .unwrap_or_else(|_| "/".to_string())
    }
}
