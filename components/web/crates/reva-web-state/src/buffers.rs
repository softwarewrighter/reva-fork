//! The file system before and after boot, and the shared byte buffer.

use crate::state::{FSBUF, PENDING, WEB};
use reva_vm::vfs::MemFs;

/// The file system: the booted host's, or the one waiting for boot.
pub fn with_fs<R>(f: impl FnOnce(&mut MemFs) -> R) -> R {
    let mut f = Some(f);
    let booted = WEB.with(|w| {
        w.borrow_mut()
            .as_mut()
            .map(|w| (f.take().unwrap())(&mut w.host.fs))
    });
    match booted {
        Some(r) => r,
        None => {
            PENDING.with(|p| (f.take().unwrap())(p.borrow_mut().get_or_insert_with(MemFs::new)))
        }
    }
}

/// The first `len` bytes of the `reva_fs_buffer` buffer.
pub fn buffer(len: u32) -> Vec<u8> {
    FSBUF.with(|b| {
        let b = b.borrow();
        b[..(len as usize).min(b.len())].to_vec()
    })
}
