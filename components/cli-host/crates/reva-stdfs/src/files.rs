//! Open files: open, close, read, write, seek.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};

use crate::stdfs::{Handle, StdFs};
use reva_hostpath::paths::errno;
use reva_vm::embedded;
use reva_vm::vfs::{self, Access, Errno, FsFiles, errno::*};

impl FsFiles for StdFs {
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
}
