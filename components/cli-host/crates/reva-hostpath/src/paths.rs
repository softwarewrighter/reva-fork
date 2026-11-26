//! Paths, errors and metadata between the host and the VM.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use reva_vm::vfs::{Errno, Stat, errno::*};

/// Where `/reva/...` lives on disk: `$REVA_HOME`, else the checkout this
/// binary was built from.
pub fn install_root() -> PathBuf {
    match std::env::var_os("REVA_HOME") {
        Some(h) => PathBuf::from(h),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.."),
    }
}

/// An `io::Error` as a Linux `errno`.
pub fn errno(e: io::Error) -> Errno {
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

/// A host file's metadata as `stat` reports it.
pub fn stat_of(m: &fs::Metadata) -> Stat {
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

/// A host path as the VM sees it: rooted at `/` and `/`-separated, which is
/// what [`vfs::normalize`] works on.  On Windows `C:\a\b` is `/C:/a/b` (and
/// a verbatim `\\?\` prefix is dropped); elsewhere the path is unchanged.
pub fn vm_path(host: &str, windows: bool) -> String {
    if !windows {
        return host.to_string();
    }
    let p = host
        .strip_prefix(r"\\?\")
        .unwrap_or(host)
        .replace('\\', "/");
    if p.starts_with('/') {
        p
    } else {
        format!("/{p}")
    }
}

/// A path a program names, as the VM's file system takes it.  On Windows
/// `\\` separates like `/`, and a drive path (`D:/a`, `C:\a`) is absolute:
/// `/D:/a` (see [`vm_path`]); elsewhere the path is unchanged.
pub fn user_path(path: &str, windows: bool) -> String {
    if !windows {
        return path.to_string();
    }
    let p = path.replace('\\', "/");
    let b = p.as_bytes();
    if b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'/' {
        format!("/{p}")
    } else {
        p
    }
}

/// The host path for a VM path: the inverse of [`vm_path`].  On Windows
/// `/C:/a/b` is `C:/a/b`, which Windows accepts; a VM path without a drive
/// (`/a/b`) stays on the current drive.
pub fn host_path(vm: &str, windows: bool) -> PathBuf {
    let b = vm.as_bytes();
    if windows && b.len() >= 3 && b[0] == b'/' && b[1].is_ascii_alphabetic() && b[2] == b':' {
        PathBuf::from(&vm[1..])
    } else {
        PathBuf::from(vm)
    }
}
