//! The newcore file words.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::vfs::Access;
use reva_words_sys::fs::with_fs;
use reva_words_sys::paths::Paths;
use reva_words_sys::sysio::SysIo;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// `open/r`, `open/rw`, `creat`, `close`, `read`, `write`, `fsize`.
    pub trait FileWords: Paths, SysIo {
        /// `PROC openr` / `openrw` / `io_create` ( a n -- fd ).
        fn word_open(&mut self, host: &mut dyn Host, access: Access) -> R {
            let n = self.upop()?;
            let a = self.eax();
            let path = self.path_at(a, n)?;
            let fd = self.sys_open(host, &path, access);
            self.set_eax(fd);
            self.set_ior(fd)
        }

        /// `PROC io_close` ( fd -- ).
        fn word_close(&mut self, host: &mut dyn Host) -> R {
            let fd = self.upop()?;
            let r = self.sys_close(host, fd);
            self.set_ior(r)
        }

        /// `PROC io_read` ( a n fd -- n' ): bytes read, or `-errno`.
        fn word_read(&mut self, host: &mut dyn Host) -> R {
            let fd = self.upop()?;
            let n = self.upop()?;
            let a = self.eax();
            let r = self.sys_read(host, fd, a, n)?;
            self.set_eax(r);
            self.set_ior(r)
        }

        /// `PROC io_write` ( a n fd -- ).
        fn word_write(&mut self, host: &mut dyn Host) -> R {
            let fd = self.upop()?;
            let n = self.upop()?;
            let a = self.upop()?;
            let r = self.sys_write(host, fd, a, n)?;
            self.set_ior(r)
        }

        /// `PROC io_size` ( fd -- size ).  Like the original, `ioerr` is left
        /// alone; a bad descriptor reads as size 0.
        fn word_fsize(&mut self, host: &mut dyn Host) -> R {
            let fd = self.eax();
            let size =
                with_fs(host, |fs| fs.fstat(fd)).map_or(0, |s| s.size.min(u32::MAX as u64) as u32);
            self.set_eax(size);
            Ok(())
        }
    }
}
