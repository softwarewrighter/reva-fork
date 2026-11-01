//! Files: put files in, collect the changed ones.

use reva_web_state::buffers::with_fs;
use reva_web_state::state::FSBUF;

/// A buffer of `len` bytes for [`reva_fs_put`].
#[unsafe(no_mangle)]
pub extern "C" fn reva_fs_buffer(len: u32) -> *mut u8 {
    FSBUF.with(|b| {
        let mut b = b.borrow_mut();
        b.resize(len as usize, 0);
        b.as_mut_ptr()
    })
}

/// The buffer's address (after [`reva_fs_changes`]).
#[unsafe(no_mangle)]
pub extern "C" fn reva_fs_data() -> *const u8 {
    FSBUF.with(|b| b.borrow().as_ptr())
}

/// How many bytes [`reva_fs_data`] holds (after [`reva_boot_image`]
/// failed, the reason).
#[unsafe(no_mangle)]
pub extern "C" fn reva_fs_len() -> u32 {
    FSBUF.with(|b| b.borrow().len() as u32)
}

/// Add the file in the buffer: a path of `path_len` bytes, then the
/// contents.  Returns 0, or -1 if `path_len` is too long.
#[unsafe(no_mangle)]
pub extern "C" fn reva_fs_put(path_len: u32) -> i32 {
    let buf = FSBUF.with(|b| std::mem::take(&mut *b.borrow_mut()));
    let n = path_len as usize;
    if n > buf.len() {
        return -1;
    }
    let path = String::from_utf8_lossy(&buf[..n]).into_owned();
    with_fs(|fs| fs.put(&path, &buf[n..]));
    0
}

/// Collect the files changed since the last call (see the module docs);
/// returns the number of bytes at [`reva_fs_data`].
#[unsafe(no_mangle)]
pub extern "C" fn reva_fs_changes() -> u32 {
    let changes = with_fs(|fs| fs.take_changes());
    FSBUF.with(|b| {
        let mut b = b.borrow_mut();
        b.clear();
        for (path, data) in changes {
            b.push(data.is_some() as u8);
            b.extend_from_slice(&(path.len() as u32).to_le_bytes());
            b.extend_from_slice(path.as_bytes());
            let data = data.unwrap_or_default();
            b.extend_from_slice(&(data.len() as u32).to_le_bytes());
            b.extend_from_slice(&data);
        }
        b.len() as u32
    })
}
