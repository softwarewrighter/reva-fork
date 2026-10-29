//! `open`/`creat`, `lseek`, `fstat`, `stat`, and the calls on a path.

use crate::fs::{SYS_CHDIR, SYS_MKDIR, SYS_UNLINK, open_access, ret, with_fs};
use crate::paths::Paths;
use crate::sysio::SysIo;
use reva_core::fault::Fault;
use reva_core::host::Host;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// System calls with their arguments as registers (`a`: EBX, ECX, ...).
    pub trait SysCalls: Paths, SysIo {
        /// `open(path, flags)` or `creat(path)`.
        fn sys_open_call(&mut self, host: &mut dyn Host, num: u32, a: &[u32; 6]) -> u32 {
            let access = open_access(num, a[1]);
            match self.zpath(a[0]) {
                Ok(p) => self.sys_open(host, &p, access),
                Err(e) => e.wrapping_neg(),
            }
        }

        /// `lseek(fd, offset, whence)`.
        fn sys_lseek(&mut self, host: &mut dyn Host, a: &[u32; 6]) -> u32 {
            let (fd, off, whence) = (a[0], a[1] as i32 as i64, a[2]);
            ret(with_fs(host, |fs| fs.seek(fd, off, whence)).map(|p| p as u32))
        }

        /// `fstat(fd, buf)`.
        fn sys_fstat(&mut self, host: &mut dyn Host, a: &[u32; 6]) -> R<u32> {
            match with_fs(host, |fs| fs.fstat(a[0])) {
                Ok(s) => self.put_stat(a[1], s),
                Err(e) => Ok(e.wrapping_neg()),
            }
        }

        /// `stat(path, buf)`.
        fn sys_stat(&mut self, host: &mut dyn Host, a: &[u32; 6]) -> R<u32> {
            match self.zpath(a[0]).and_then(|p| with_fs(host, |fs| fs.stat(&p))) {
                Ok(s) => self.put_stat(a[1], s),
                Err(e) => Ok(e.wrapping_neg()),
            }
        }

        /// `unlink`, `chdir`, `mkdir`, `rmdir` of a path.
        fn sys_path(&mut self, host: &mut dyn Host, num: u32, a: &[u32; 6]) -> u32 {
            let r = self.zpath(a[0]).and_then(|p| {
                with_fs(host, |fs| match num {
                    SYS_UNLINK => fs.remove(&p),
                    SYS_CHDIR => fs.chdir(&p),
                    SYS_MKDIR => fs.mkdir(&p),
                    _ => fs.rmdir(&p),
                })
            });
            ret(r.map(|()| 0))
        }
    }
}
