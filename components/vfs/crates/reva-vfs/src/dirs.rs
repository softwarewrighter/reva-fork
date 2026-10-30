//! Directories, whole files, and the file system as one trait.

use crate::io::FsFiles;
use crate::meta::FsMeta;
use crate::types::{Access, Errno};

/// Directories and the working directory; whole-file reads.
pub trait FsDirs: FsFiles {
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

/// The operations behind `reva.f`'s file words: every file system that
/// has the three parts is one.
pub trait FileSystem: FsFiles + FsMeta + FsDirs {}

impl<T: FsFiles + FsMeta + FsDirs> FileSystem for T {}
