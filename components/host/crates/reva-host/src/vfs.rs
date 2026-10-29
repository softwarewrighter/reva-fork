//! # Files: the file system a host lends the core
//!
//! `reva.f`'s file words (`open/r`, `read`, `seek`, `stat`, `rename`, ...)
//! end in a handful of Linux system calls.  The core implements those calls
//! (see `fileio.rs`) over [`FileSystem`], which a host provides through
//! [`crate::Host::fs`]:
//!
//! * the CLI lends the real file system (`reva-cli`'s `StdFs`);
//! * the browser and tests lend [`MemFs`], an in-memory tree that records
//!   which paths changed, so a page can persist them (e.g. in
//!   `localStorage`) and restore them on the next visit.
//!
//! Both see the library tree `lib/` at `/reva/lib/` even when nothing is
//! on disk: it is compiled into the core ([`crate::embedded::lib_file`]),
//! read-only, so `needs` works on every host.  A file a program writes
//! there shadows the embedded copy.
//!
//! Errors are Linux `errno` values ([`errno`]), because that is what
//! `reva.f` and the libraries test for (`ioerr` holds `-errno`).

use std::collections::{BTreeMap, BTreeSet};

use crate::embedded;

/// A positive Linux `errno` value.
pub type Errno = u32;

/// The `errno` values the file layer produces (Linux numbering).
pub mod errno {
    use super::Errno;
    pub const ENOENT: Errno = 2;
    pub const EBADF: Errno = 9;
    pub const EFAULT: Errno = 14;
    pub const EACCES: Errno = 13;
    pub const EBUSY: Errno = 16;
    pub const EEXIST: Errno = 17;
    pub const ENOTDIR: Errno = 20;
    pub const EISDIR: Errno = 21;
    pub const EINVAL: Errno = 22;
    pub const ERANGE: Errno = 34;
    pub const ENOSYS: Errno = 38;
    pub const ENOTEMPTY: Errno = 39;
}
use errno::*;

/// How a file is opened: `open/r`, `open/rw`, or `creat` (create or
/// truncate, write-only — as Linux `creat(2)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Read,
    ReadWrite,
    Create,
}

impl Access {
    pub fn readable(self) -> bool {
        self != Access::Create
    }
    pub fn writable(self) -> bool {
        self != Access::Read
    }
}

/// What `stat` reports.  `mtime` is seconds since the Unix epoch (0 when
/// the host has no clock).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stat {
    pub size: u64,
    pub dir: bool,
    pub mtime: u64,
}

pub const SEEK_SET: u32 = 0;
pub const SEEK_CUR: u32 = 1;
pub const SEEK_END: u32 = 2;

/// File descriptors 0-2 are the console (handled by the core); a
/// file system numbers its own from here.
pub const FIRST_FD: u32 = 3;

/// The operations behind `reva.f`'s file words.  Paths are as Forth code
/// wrote them: absolute, or relative to [`FileSystem::cwd`].
pub trait FileSystem {
    fn open(&mut self, path: &str, access: Access) -> Result<u32, Errno>;
    fn close(&mut self, fd: u32) -> Result<(), Errno>;
    fn read(&mut self, fd: u32, buf: &mut [u8]) -> Result<usize, Errno>;
    fn write(&mut self, fd: u32, data: &[u8]) -> Result<usize, Errno>;
    /// `lseek`: the new position.
    fn seek(&mut self, fd: u32, offset: i64, whence: u32) -> Result<u64, Errno>;
    fn fstat(&mut self, fd: u32) -> Result<Stat, Errno>;
    fn stat(&mut self, path: &str) -> Result<Stat, Errno>;
    /// `unlink`.
    fn remove(&mut self, path: &str) -> Result<(), Errno>;
    fn rename(&mut self, from: &str, to: &str) -> Result<(), Errno>;
    fn mkdir(&mut self, path: &str) -> Result<(), Errno>;
    fn rmdir(&mut self, path: &str) -> Result<(), Errno>;
    /// Names in a directory, without `.` and `..`, in any order.
    fn list_dir(&mut self, path: &str) -> Result<Vec<String>, Errno>;
    fn chdir(&mut self, path: &str) -> Result<(), Errno>;
    fn cwd(&mut self) -> String;

    /// A whole file, for `include` and `slurp`.
    fn read_whole(&mut self, path: &str) -> Result<Vec<u8>, Errno> {
        let fd = self.open(path, Access::Read)?;
        let mut data = Vec::new();
        let mut buf = vec![0u8; 64 * 1024];
        let r = loop {
            match self.read(fd, &mut buf) {
                Ok(0) => break Ok(()),
                Ok(n) => data.extend_from_slice(&buf[..n]),
                Err(e) => break Err(e),
            }
        };
        let _ = self.close(fd);
        r.map(|()| data)
    }
}

/// `path` made absolute against `cwd`, with `.`, `..` and repeated
/// slashes removed (`/` for the root).
pub fn normalize(cwd: &str, path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    let joined;
    let full = if path.starts_with('/') {
        path
    } else {
        joined = format!("{cwd}/{path}");
        &joined
    };
    for p in full.split('/') {
        match p {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    format!("/{}", parts.join("/"))
}

fn parent(path: &str) -> &str {
    match path.rfind('/') {
        Some(0) | None => "/",
        Some(i) => &path[..i],
    }
}

/// A change [`MemFs::take_changes`] reports: the file's new contents, or
/// `None` if it was removed.
pub type Change = (String, Option<Vec<u8>>);

struct OpenFile {
    path: String,
    pos: u64,
    access: Access,
}

/// An in-memory file system over the embedded library tree.
///
/// Directories are implicit (any prefix of a file path) or made with
/// `mkdir`; only files are reported as changes, so an empty directory does
/// not outlive the page.  Files under the embedded tree are read-only
/// until written: the first write copies them.
pub struct MemFs {
    files: BTreeMap<String, Vec<u8>>,
    dirs: BTreeSet<String>,
    open: BTreeMap<u32, OpenFile>,
    cwd: String,
    changed: BTreeSet<String>,
    /// Seconds since the epoch stamped on writes (see [`MemFs::set_time`]).
    time: u64,
    mtimes: BTreeMap<String, u64>,
}

impl Default for MemFs {
    fn default() -> MemFs {
        MemFs::new()
    }
}

impl MemFs {
    pub fn new() -> MemFs {
        MemFs {
            files: BTreeMap::new(),
            dirs: BTreeSet::new(),
            open: BTreeMap::new(),
            cwd: "/".to_string(),
            changed: BTreeSet::new(),
            time: 0,
            mtimes: BTreeMap::new(),
        }
    }

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

    /// Whether [`MemFs::take_changes`] has anything to report.
    pub fn has_changes(&self) -> bool {
        !self.changed.is_empty()
    }

    /// The clock for modification times (the core itself has none).
    pub fn set_time(&mut self, secs: u64) {
        self.time = secs;
    }

    fn abs(&self, path: &str) -> String {
        normalize(&self.cwd, path)
    }

    fn data(&self, p: &str) -> Option<&[u8]> {
        self.files
            .get(p)
            .map(Vec::as_slice)
            .or_else(|| embedded::lib_file(p))
    }

    fn is_file(&self, p: &str) -> bool {
        self.data(p).is_some()
    }

    fn is_dir(&self, p: &str) -> bool {
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

    fn touch(&mut self, p: &str) {
        self.changed.insert(p.to_string());
        self.mtimes.insert(p.to_string(), self.time);
    }

    /// Make an embedded file writable by copying it.
    fn materialize(&mut self, p: &str) {
        if !self.files.contains_key(p) {
            let data = embedded::lib_file(p)
                .map(<[u8]>::to_vec)
                .unwrap_or_default();
            self.files.insert(p.to_string(), data);
        }
    }

    fn handle(&mut self, fd: u32) -> Result<&mut OpenFile, Errno> {
        self.open.get_mut(&fd).ok_or(EBADF)
    }

    fn stat_abs(&self, p: &str) -> Result<Stat, Errno> {
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

impl FileSystem for MemFs {
    fn open(&mut self, path: &str, access: Access) -> Result<u32, Errno> {
        let p = self.abs(path);
        if self.is_dir(&p) {
            return Err(EISDIR);
        }
        if access == Access::Create {
            if !self.is_dir(parent(&p)) {
                return Err(ENOENT);
            }
            self.files.insert(p.clone(), Vec::new());
            self.touch(&p);
        } else if !self.is_file(&p) {
            return Err(ENOENT);
        }
        let fd = (FIRST_FD..)
            .find(|f| !self.open.contains_key(f))
            .ok_or(EBADF)?;
        self.open.insert(
            fd,
            OpenFile {
                path: p,
                pos: 0,
                access,
            },
        );
        Ok(fd)
    }

    fn close(&mut self, fd: u32) -> Result<(), Errno> {
        self.open.remove(&fd).map(drop).ok_or(EBADF)
    }

    fn read(&mut self, fd: u32, buf: &mut [u8]) -> Result<usize, Errno> {
        let f = self.handle(fd)?;
        if !f.access.readable() {
            return Err(EBADF);
        }
        let (path, pos) = (f.path.clone(), f.pos);
        let data = self.data(&path).unwrap_or(&[]);
        let start = (pos as usize).min(data.len());
        let n = buf.len().min(data.len() - start);
        buf[..n].copy_from_slice(&data[start..start + n]);
        self.handle(fd)?.pos += n as u64;
        Ok(n)
    }

    fn write(&mut self, fd: u32, bytes: &[u8]) -> Result<usize, Errno> {
        let f = self.handle(fd)?;
        if !f.access.writable() {
            return Err(EBADF);
        }
        let (path, pos) = (f.path.clone(), f.pos as usize);
        self.materialize(&path);
        let data = self.files.get_mut(&path).ok_or(EBADF)?;
        if data.len() < pos + bytes.len() {
            data.resize(pos + bytes.len(), 0);
        }
        data[pos..pos + bytes.len()].copy_from_slice(bytes);
        self.touch(&path);
        self.handle(fd)?.pos += bytes.len() as u64;
        Ok(bytes.len())
    }

    fn seek(&mut self, fd: u32, offset: i64, whence: u32) -> Result<u64, Errno> {
        let f = self.handle(fd)?;
        let path = f.path.clone();
        let base = match whence {
            SEEK_SET => 0,
            SEEK_CUR => f.pos as i64,
            SEEK_END => self.data(&path).map_or(0, |d| d.len() as i64),
            _ => return Err(EINVAL),
        };
        let pos = base.checked_add(offset).filter(|p| *p >= 0).ok_or(EINVAL)?;
        self.handle(fd)?.pos = pos as u64;
        Ok(pos as u64)
    }

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

    fn rename(&mut self, from: &str, to: &str) -> Result<(), Errno> {
        let (a, b) = (self.abs(from), self.abs(to));
        if !self.is_dir(parent(&b)) {
            return Err(ENOENT);
        }
        if self.files.contains_key(&a) {
            if self.is_dir(&b) {
                return Err(EISDIR);
            }
            let data = self.files.remove(&a).unwrap_or_default();
            self.files.insert(b.clone(), data);
            self.mtimes.remove(&a);
            self.changed.insert(a);
            self.touch(&b);
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
        // a directory: move everything below it
        let prefix = format!("{a}/");
        let moved: Vec<String> = self
            .files
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();
        for k in moved {
            let data = self.files.remove(&k).unwrap_or_default();
            let nk = format!("{b}{}", &k[a.len()..]);
            self.files.insert(nk.clone(), data);
            self.changed.insert(k);
            self.touch(&nk);
        }
        let dirs: Vec<String> = self
            .dirs
            .iter()
            .filter(|k| **k == a || k.starts_with(&prefix))
            .cloned()
            .collect();
        for k in dirs {
            self.dirs.remove(&k);
            self.dirs.insert(format!("{b}{}", &k[a.len()..]));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_normalize() {
        assert_eq!(normalize("/", "a/b"), "/a/b");
        assert_eq!(normalize("/x/y", "../z/./w"), "/x/z/w");
        assert_eq!(normalize("/x", "/a//b/"), "/a/b");
        assert_eq!(normalize("/", ".."), "/");
    }

    #[test]
    fn memfs_round_trip() {
        let mut fs = MemFs::new();
        let fd = fs.open("/t.txt", Access::Create).unwrap();
        assert_eq!(fs.write(fd, b"hello").unwrap(), 5);
        assert_eq!(fs.read(fd, &mut [0; 4]), Err(EBADF)); // creat is write-only
        fs.close(fd).unwrap();
        let fd = fs.open("t.txt", Access::ReadWrite).unwrap();
        assert_eq!(fs.seek(fd, 0, SEEK_END), Ok(5));
        fs.write(fd, b" world").unwrap();
        assert_eq!(fs.seek(fd, 6, SEEK_SET), Ok(6));
        let mut buf = [0; 16];
        assert_eq!(fs.read(fd, &mut buf), Ok(5));
        assert_eq!(&buf[..5], b"world");
        assert_eq!(fs.fstat(fd).unwrap().size, 11);
        fs.close(fd).unwrap();
        assert_eq!(fs.close(fd), Err(EBADF));
        assert_eq!(
            fs.take_changes(),
            vec![("/t.txt".into(), Some(b"hello world".to_vec()))]
        );
        assert!(!fs.has_changes());
    }

    #[test]
    fn memfs_dirs_and_names() {
        let mut fs = MemFs::new();
        assert_eq!(fs.open("/d/x", Access::Create), Err(ENOENT));
        fs.mkdir("/d").unwrap();
        assert_eq!(fs.mkdir("/d"), Err(EEXIST));
        fs.chdir("/d").unwrap();
        let fd = fs.open("x", Access::Create).unwrap();
        fs.close(fd).unwrap();
        assert_eq!(fs.list_dir(".").unwrap(), vec!["x".to_string()]);
        assert_eq!(fs.rmdir("/d"), Err(ENOTEMPTY));
        fs.rename("x", "/y").unwrap();
        assert_eq!(fs.stat("/y").unwrap().size, 0);
        assert_eq!(fs.stat("/d/x"), Err(ENOENT));
        fs.rmdir("/d").unwrap();
        fs.remove("/y").unwrap();
        assert_eq!(fs.remove("/y"), Err(ENOENT));
        let changes = fs.take_changes();
        assert!(changes.iter().all(|(_, d)| d.is_none()));
    }

    #[cfg(feature = "embed-lib")]
    #[test]
    fn memfs_sees_the_embedded_library() {
        let mut fs = MemFs::new();
        assert!(fs.stat("/reva/lib").unwrap().dir);
        assert!(
            fs.list_dir("/reva/lib/util")
                .unwrap()
                .contains(&"locals".to_string())
        );
        assert!(fs.list_dir("/").unwrap().contains(&"reva".to_string()));
        let src = fs.read_whole("/reva/lib/util/locals").unwrap();
        assert!(!src.is_empty());
        assert_eq!(fs.remove("/reva/lib/util/locals"), Err(EACCES));
        // writing copies it; the copy shadows the embedded one
        let fd = fs.open("/reva/lib/util/locals", Access::ReadWrite).unwrap();
        fs.write(fd, b"|").unwrap();
        fs.close(fd).unwrap();
        assert_eq!(fs.get("/reva/lib/util/locals").unwrap()[0], b'|');
    }
}
