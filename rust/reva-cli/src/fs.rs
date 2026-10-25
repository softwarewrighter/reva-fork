//! The real file system, for `reva.f`'s file words on the command line.
//!
//! Paths are resolved against the process's working directory (which
//! `chdir` changes).  The core's virtual install prefix `/reva/` maps to
//! [`install_root`]; below it, `lib/` falls back to the copy compiled into
//! the core when the file is not on disk, so an installed `reva` binary
//! runs `needs` without a checkout.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use reva_vm::embedded;
use reva_vm::vfs::{self, errno::*, Access, Errno, FileSystem, Stat, FIRST_FD};

/// Where `/reva/...` lives on disk: `$REVA_HOME`, else the checkout this
/// binary was built from.
pub fn install_root() -> PathBuf {
    match std::env::var_os("REVA_HOME") {
        Some(h) => PathBuf::from(h),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
    }
}

/// An `io::Error` as a Linux `errno`.
fn errno(e: io::Error) -> Errno {
    #[cfg(target_os = "linux")]
    if let Some(n) = e.raw_os_error() {
        return n as Errno;
    }
    match e.kind() {
        io::ErrorKind::NotFound => ENOENT,
        io::ErrorKind::PermissionDenied => EACCES,
        io::ErrorKind::AlreadyExists => EEXIST,
        _ => EINVAL,
    }
}

enum Handle {
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
pub struct StdFs {
    open: BTreeMap<u32, Handle>,
}

impl StdFs {
    /// (virtual absolute path, host path)
    fn resolve(&mut self, path: &str) -> (String, PathBuf) {
        let v = vfs::normalize(&self.cwd(), path);
        let host = match embedded::install_relative(&v) {
            Some(rel) => install_root().join(rel),
            None => PathBuf::from(&v),
        };
        (v, host)
    }

    fn insert(&mut self, h: Handle) -> u32 {
        let fd = (FIRST_FD..)
            .find(|f| !self.open.contains_key(f))
            .unwrap_or(u32::MAX);
        self.open.insert(fd, h);
        fd
    }

    fn handle(&mut self, fd: u32) -> Result<&mut Handle, Errno> {
        self.open.get_mut(&fd).ok_or(EBADF)
    }
}

fn stat_of(m: &fs::Metadata) -> Stat {
    let mtime = m
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    Stat {
        size: m.len(),
        dir: m.is_dir(),
        mtime,
    }
}

impl FileSystem for StdFs {
    fn open(&mut self, path: &str, access: Access) -> Result<u32, Errno> {
        let (v, p) = self.resolve(path);
        if p.is_dir() {
            return Err(EISDIR);
        }
        let file = match access {
            Access::Read => File::open(&p),
            Access::ReadWrite => File::options().read(true).write(true).open(&p),
            Access::Create => File::create(&p),
        };
        match file {
            Ok(file) => Ok(self.insert(Handle::Disk { file, access })),
            Err(e) if e.kind() == io::ErrorKind::NotFound && access == Access::Read => {
                let data = embedded::lib_file(&v).ok_or(ENOENT)?;
                Ok(self.insert(Handle::Embedded { data, pos: 0 }))
            }
            Err(e) => Err(errno(e)),
        }
    }

    fn close(&mut self, fd: u32) -> Result<(), Errno> {
        self.open.remove(&fd).map(drop).ok_or(EBADF)
    }

    fn read(&mut self, fd: u32, buf: &mut [u8]) -> Result<usize, Errno> {
        match self.handle(fd)? {
            Handle::Disk {
                access: Access::Create,
                ..
            } => Err(EBADF),
            Handle::Disk { file, .. } => file.read(buf).map_err(errno),
            Handle::Embedded { data, pos } => {
                let start = (*pos as usize).min(data.len());
                let n = buf.len().min(data.len() - start);
                buf[..n].copy_from_slice(&data[start..start + n]);
                *pos += n as u64;
                Ok(n)
            }
        }
    }

    fn write(&mut self, fd: u32, data: &[u8]) -> Result<usize, Errno> {
        match self.handle(fd)? {
            Handle::Disk {
                access: Access::Read,
                ..
            }
            | Handle::Embedded { .. } => Err(EBADF),
            Handle::Disk { file, .. } => file.write_all(data).map(|()| data.len()).map_err(errno),
        }
    }

    fn seek(&mut self, fd: u32, offset: i64, whence: u32) -> Result<u64, Errno> {
        let to = match whence {
            vfs::SEEK_SET => SeekFrom::Start(u64::try_from(offset).map_err(|_| EINVAL)?),
            vfs::SEEK_CUR => SeekFrom::Current(offset),
            vfs::SEEK_END => SeekFrom::End(offset),
            _ => return Err(EINVAL),
        };
        match self.handle(fd)? {
            Handle::Disk { file, .. } => file.seek(to).map_err(errno),
            Handle::Embedded { data, pos } => {
                let base = match to {
                    SeekFrom::Start(_) => 0,
                    SeekFrom::Current(_) => *pos as i64,
                    SeekFrom::End(_) => data.len() as i64,
                };
                let p = base.checked_add(offset).filter(|p| *p >= 0).ok_or(EINVAL)?;
                *pos = p as u64;
                Ok(*pos)
            }
        }
    }

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
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "/".to_string())
    }
}
