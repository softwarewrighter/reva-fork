//! The file system calls of `os_syscall`, by number.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::vfs::errno::{EFAULT, ERANGE};
use reva_words_sys::fs::*;
use reva_words_sys::paths::Paths;
use reva_words_sys::syscalls::SysCalls;
use reva_words_sys::sysio::SysIo;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// File-related Linux system calls.
    pub trait SysFile: Paths, SysIo, SysCalls {
        /// File-related Linux system calls for `os_syscall`; `None` for any
        /// other number.  `a` holds EBX, ECX, EDX, ... in order.
        fn sys_file(&mut self, host: &mut dyn Host, num: u32, a: &[u32; 6]) -> R<Option<u32>> {
            let r = match num {
                SYS_READ => self.sys_read(host, a[0], a[1], a[2])?,
                SYS_WRITE => self.sys_write(host, a[0], a[1], a[2])?,
                SYS_OPEN | SYS_CREAT => self.sys_open_call(host, num, a),
                SYS_CLOSE => self.sys_close(host, a[0]),
                SYS_LSEEK => self.sys_lseek(host, a),
                SYS_FSTAT => self.sys_fstat(host, a)?,
                SYS_STAT => self.sys_stat(host, a)?,
                SYS_UNLINK | SYS_CHDIR | SYS_MKDIR | SYS_RMDIR => self.sys_path(host, num, a),
                SYS_RENAME => self.sys_rename(host, a),
                SYS_GETCWD => self.sys_getcwd(host, a)?,
                _ => return Ok(None),
            };
            Ok(Some(r))
        }

        /// `rename(from, to)`.
        fn sys_rename(&mut self, host: &mut dyn Host, a: &[u32; 6]) -> u32 {
            let r = self.zpath(a[0]).and_then(|from| {
                let to = self.zpath(a[1])?;
                with_fs(host, |fs| fs.rename(&from, &to))
            });
            ret(r.map(|()| 0))
        }

        /// `getcwd(buf, size)`: the length with the terminating 0.
        fn sys_getcwd(&mut self, host: &mut dyn Host, a: &[u32; 6]) -> R<u32> {
            let (buf, size) = (a[0], a[1]);
            let cwd = match with_fs(host, |fs| Ok(fs.cwd())) {
                Ok(cwd) => cwd,
                Err(e) => return Ok(e.wrapping_neg()),
            };
            let n = cwd.len() as u32 + 1;
            if n > size {
                return Ok(ERANGE.wrapping_neg());
            }
            if self.mem.slice(buf, n).is_err() {
                return Ok(EFAULT.wrapping_neg());
            }
            self.mem.write_bytes(buf, cwd.as_bytes())?;
            self.mem.write_u8(buf + n - 1, 0)?;
            Ok(n)
        }
    }
}
