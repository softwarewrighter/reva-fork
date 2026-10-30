//! Open files: open, close, read, write, seek.

use crate::types::{Access, Errno};

/// The descriptor operations.  Paths are as Forth code wrote them:
/// absolute, or relative to [`crate::dirs::FsDirs::cwd`].
pub trait FsFiles {
    fn open(&mut self, path: &str, access: Access) -> Result<u32, Errno>;
    fn close(&mut self, fd: u32) -> Result<(), Errno>;
    fn read(&mut self, fd: u32, buf: &mut [u8]) -> Result<usize, Errno>;
    fn write(&mut self, fd: u32, data: &[u8]) -> Result<usize, Errno>;
    /// `lseek`: the new position.
    fn seek(&mut self, fd: u32, offset: i64, whence: u32) -> Result<u64, Errno>;
}
