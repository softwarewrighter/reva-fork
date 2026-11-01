//! System call numbers, flags, and the result convention.

use reva_core::host::Host;
use reva_core::vfs::{Access, Errno, FileSystem, errno::ENOSYS};

// Linux i386 system call numbers.
pub const SYS_READ: u32 = 3;
pub const SYS_WRITE: u32 = 4;
pub const SYS_OPEN: u32 = 5;
pub const SYS_CLOSE: u32 = 6;
pub const SYS_CREAT: u32 = 8;
pub const SYS_UNLINK: u32 = 10;
pub const SYS_CHDIR: u32 = 12;
pub const SYS_LSEEK: u32 = 19;
pub const SYS_RENAME: u32 = 38;
pub const SYS_MKDIR: u32 = 39;
pub const SYS_RMDIR: u32 = 40;
pub const SYS_STAT: u32 = 106;
pub const SYS_FSTAT: u32 = 108;
pub const SYS_GETCWD: u32 = 183;

// open(2) flags (i386).
const O_ACCMODE: u32 = 3;
const O_CREAT: u32 = 0o100;
const O_TRUNC: u32 = 0o1000;

/// The kernel's result convention: the value, or `-errno`.
pub fn ret(r: Result<u32, Errno>) -> u32 {
    r.unwrap_or_else(|e| e.wrapping_neg())
}

/// Run `f` on the host's file system; `ENOSYS` if it has none.
pub fn with_fs<T>(
    host: &mut dyn Host,
    f: impl FnOnce(&mut dyn FileSystem) -> Result<T, Errno>,
) -> Result<T, Errno> {
    host.fs().map_or(Err(ENOSYS), f)
}

/// The access `open` (with `flags`) or `creat` asks for.
pub fn open_access(num: u32, flags: u32) -> Access {
    if num == SYS_CREAT || flags & (O_CREAT | O_TRUNC) != 0 {
        Access::Create
    } else if flags & O_ACCMODE == 0 {
        Access::Read
    } else {
        Access::ReadWrite
    }
}
