//! Open files in memory.

use crate::memfs::{MemFs, OpenFile};
use reva_vfs::io::FsFiles;
use reva_vfs::types::{Access, Errno, FIRST_FD, SEEK_CUR, SEEK_END, SEEK_SET, errno::*, parent};

impl FsFiles for MemFs {
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
}
