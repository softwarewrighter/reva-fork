//! `open`, `close`, `read`, `write`, and `struct stat`.

use crate::fs::{ret, with_fs};
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::vfs::{Access, FIRST_FD, Stat, errno::*};

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// The descriptor calls.
    pub trait SysIo {
        /// Open `path`: a descriptor, or `-errno`.
        fn sys_open(&mut self, host: &mut dyn Host, path: &str, access: Access) -> u32 {
            ret(with_fs(host, |fs| fs.open(path, access)))
        }

        /// Close `fd` (the console's descriptors stay open).
        fn sys_close(&mut self, host: &mut dyn Host, fd: u32) -> u32 {
            if fd < FIRST_FD {
                return 0;
            }
            ret(with_fs(host, |fs| fs.close(fd).map(|()| 0)))
        }

        /// Read up to `n` bytes from `fd` into `a`.
        fn sys_read(&mut self, host: &mut dyn Host, fd: u32, a: u32, n: u32) -> R<u32> {
            if fd < FIRST_FD {
                return Ok(EBADF.wrapping_neg());
            }
            if self.mem.slice(a, n).is_err() {
                return Ok(EFAULT.wrapping_neg());
            }
            let mut buf = vec![0u8; n as usize];
            let r = with_fs(host, |fs| fs.read(fd, &mut buf));
            if let Ok(k) = r {
                self.mem.write_bytes(a, &buf[..k])?;
            }
            Ok(ret(r.map(|k| k as u32)))
        }

        /// Write `n` bytes at `a` to `fd` (1 and 2: the console).
        fn sys_write(&mut self, host: &mut dyn Host, fd: u32, a: u32, n: u32) -> R<u32> {
            let Ok(data) = self.mem.slice(a, n) else {
                return Ok(EFAULT.wrapping_neg());
            };
            let data = data.to_vec();
            Ok(match fd {
                1 | 2 => {
                    host.write(&data);
                    n
                }
                0 => EBADF.wrapping_neg(),
                _ => ret(with_fs(host, |fs| fs.write(fd, &data)).map(|k| k as u32)),
            })
        }

        /// `struct stat` (i386 `sys_newstat`, 64 bytes) at `buf`.  `reva.f`
        /// reads the cell at offset 8 for `stat` (mode, nlink) and the one at
        /// 48 for `mtime` (the kernel's ctime), so ctime repeats mtime.
        fn put_stat(&mut self, buf: u32, s: Stat) -> R<u32> {
            if self.mem.slice(buf, 64).is_err() {
                return Ok(EFAULT.wrapping_neg());
            }
            self.mem.fill(buf, 64, 0)?;
            let mode: u16 = if s.dir { 0o040755 } else { 0o100644 };
            let size = s.size.min(u32::MAX as u64) as u32;
            let mtime = s.mtime as u32;
            self.mem.write_u32(buf + 4, 1)?; // st_ino
            self.mem.write_u16(buf + 8, mode)?;
            self.mem.write_u16(buf + 10, 1)?; // st_nlink
            self.mem.write_u32(buf + 20, size)?;
            self.mem.write_u32(buf + 24, 4096)?; // st_blksize
            self.mem.write_u32(buf + 28, size.div_ceil(512))?;
            for off in [32, 40, 48] {
                self.mem.write_u32(buf + off, mtime)?;
            }
            Ok(0)
        }
    }
}
