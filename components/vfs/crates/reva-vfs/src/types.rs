//! # Files: the file system a host lends the core
//!
//! `reva.f`'s file words (`open/r`, `read`, `seek`, `stat`, `rename`, ...)
//! end in a handful of Linux system calls.  The core implements those calls
//! (reva-words-os) over the traits of this crate ([`crate::dirs::FileSystem`]),
//! which a host provides through `Host::fs` (reva-host):
//!
//! * the CLI lends the real file system (`reva-cli`'s `StdFs`);
//! * the browser and tests lend `MemFs` (reva-memfs), an in-memory tree that records
//!   which paths changed, so a page can persist them (e.g. in
//!   `localStorage`) and restore them on the next visit.
//!
//! Both see the library tree `lib/` at `/reva/lib/` even when nothing is
//! on disk: it is compiled into the core (reva-embedded's `lib_file`),
//! read-only, so `needs` works on every host.  A file a program writes
//! there shadows the embedded copy.
//!
//! Errors are Linux `errno` values ([`errno`]), because that is what
//! `reva.f` and the libraries test for (`ioerr` holds `-errno`).

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

/// The directory part of an absolute path (`/` for the root).
pub fn parent(path: &str) -> &str {
    match path.rfind('/') {
        Some(0) | None => "/",
        Some(i) => &path[..i],
    }
}
