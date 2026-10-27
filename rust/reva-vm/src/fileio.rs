//! # File words and file system calls, over [`Host::fs`]
//!
//! The newcore file words (`open/r`, `open/rw`, `creat`, `close`, `read`,
//! `write`, `fsize`) keep their stack effects and their `ioerr` rule: 0 on
//! success, the negative `errno` on failure, as the original's
//! `call_linux_return` left it.  `reva.f`'s other file words are Linux
//! system calls (`seek`, `rename`, `delete`, `stat`, `chdir`, `getcwd`,
//! and `mkdir`/`rmdir` in `lib/os/dir`); [`Machine::sys_file`] serves those
//! numbers with the kernel's register order and result convention, and
//! the virtual libc gains `opendir`/`readdir`/`closedir` for `lib/os/dir`.
//!
//! File descriptors 0-2 stay the console: writing to 1 or 2 is output,
//! closing them does nothing, and reading 0 fails with `EBADF` (input
//! reaches Forth through `key`, which can pause the VM; `read` cannot).

use crate::fault::Fault;
use crate::host::Host;
use crate::machine::{Machine, st};
use crate::vfs::{Access, Errno, FIRST_FD, FileSystem, Stat, errno::*};

type R<T = ()> = Result<T, Fault>;

// Linux i386 system call numbers.
pub(crate) const SYS_READ: u32 = 3;
pub(crate) const SYS_WRITE: u32 = 4;
pub(crate) const SYS_OPEN: u32 = 5;
pub(crate) const SYS_CLOSE: u32 = 6;
pub(crate) const SYS_CREAT: u32 = 8;
pub(crate) const SYS_UNLINK: u32 = 10;
pub(crate) const SYS_CHDIR: u32 = 12;
pub(crate) const SYS_LSEEK: u32 = 19;
pub(crate) const SYS_RENAME: u32 = 38;
pub(crate) const SYS_MKDIR: u32 = 39;
pub(crate) const SYS_RMDIR: u32 = 40;
pub(crate) const SYS_STAT: u32 = 106;
pub(crate) const SYS_FSTAT: u32 = 108;
pub(crate) const SYS_GETCWD: u32 = 183;

// open(2) flags (i386).
const O_ACCMODE: u32 = 3;
const O_CREAT: u32 = 0o100;
const O_TRUNC: u32 = 0o1000;

/// `opendir` handles: opaque, nonzero, never a valid address.
const DIR_HANDLE_BASE: u32 = 0x4449_0000; // "DI"
const DIRENT_SIZE: u32 = 11 + 256;

/// A directory being read by `readdir`.
pub(crate) struct DirStream {
    path: String,
    names: Vec<String>,
    next: usize,
}

/// The kernel's result convention: the value, or `-errno`.
fn ret(r: Result<u32, Errno>) -> u32 {
    r.unwrap_or_else(|e| e.wrapping_neg())
}

fn with_fs<T>(
    host: &mut dyn Host,
    f: impl FnOnce(&mut dyn FileSystem) -> Result<T, Errno>,
) -> Result<T, Errno> {
    host.fs().map_or(Err(ENOSYS), f)
}

impl Machine {
    fn path_at(&self, a: u32, n: u32) -> R<String> {
        Ok(String::from_utf8_lossy(self.mem.slice(a, n)?).into_owned())
    }

    /// A zero-terminated path, or `EFAULT` if it runs off memory.
    fn zpath(&self, a: u32) -> Result<String, Errno> {
        let mut n = 0;
        loop {
            match self.mem.read_u8(a.wrapping_add(n)) {
                Ok(0) => break,
                Ok(_) => n += 1,
                Err(_) => return Err(EFAULT),
            }
        }
        self.path_at(a, n).map_err(|_| EFAULT)
    }

    /// `ioerr` after a call: 0, or the negative result.
    fn set_ior(&mut self, result: u32) -> R {
        let v = if (result as i32) < 0 { result } else { 0 };
        self.setv(st::IOR, v)
    }

    fn sys_open(&mut self, host: &mut dyn Host, path: &str, access: Access) -> u32 {
        ret(with_fs(host, |fs| fs.open(path, access)))
    }

    fn sys_close(&mut self, host: &mut dyn Host, fd: u32) -> u32 {
        if fd < FIRST_FD {
            return 0;
        }
        ret(with_fs(host, |fs| fs.close(fd).map(|()| 0)))
    }

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

    /// File-related Linux system calls for `os_syscall`; `None` for any
    /// other number.  `a` holds EBX, ECX, EDX, ... in order.
    pub(crate) fn sys_file(
        &mut self,
        host: &mut dyn Host,
        num: u32,
        a: &[u32; 6],
    ) -> R<Option<u32>> {
        let r = match num {
            SYS_READ => self.sys_read(host, a[0], a[1], a[2])?,
            SYS_WRITE => self.sys_write(host, a[0], a[1], a[2])?,
            SYS_OPEN | SYS_CREAT => {
                let access = if num == SYS_CREAT || a[1] & (O_CREAT | O_TRUNC) != 0 {
                    Access::Create
                } else if a[1] & O_ACCMODE == 0 {
                    Access::Read
                } else {
                    Access::ReadWrite
                };
                match self.zpath(a[0]) {
                    Ok(p) => self.sys_open(host, &p, access),
                    Err(e) => e.wrapping_neg(),
                }
            }
            SYS_CLOSE => self.sys_close(host, a[0]),
            SYS_LSEEK => {
                let (fd, off, whence) = (a[0], a[1] as i32 as i64, a[2]);
                ret(with_fs(host, |fs| fs.seek(fd, off, whence)).map(|p| p as u32))
            }
            SYS_FSTAT => match with_fs(host, |fs| fs.fstat(a[0])) {
                Ok(s) => self.put_stat(a[1], s)?,
                Err(e) => e.wrapping_neg(),
            },
            SYS_STAT => match self
                .zpath(a[0])
                .and_then(|p| with_fs(host, |fs| fs.stat(&p)))
            {
                Ok(s) => self.put_stat(a[1], s)?,
                Err(e) => e.wrapping_neg(),
            },
            SYS_UNLINK | SYS_CHDIR | SYS_MKDIR | SYS_RMDIR => {
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
            SYS_RENAME => {
                let r = self.zpath(a[0]).and_then(|from| {
                    let to = self.zpath(a[1])?;
                    with_fs(host, |fs| fs.rename(&from, &to))
                });
                ret(r.map(|()| 0))
            }
            SYS_GETCWD => {
                let (buf, size) = (a[0], a[1]);
                match with_fs(host, |fs| Ok(fs.cwd())) {
                    Ok(cwd) => {
                        let n = cwd.len() as u32 + 1;
                        if n > size {
                            ERANGE.wrapping_neg()
                        } else if self.mem.slice(buf, n).is_err() {
                            EFAULT.wrapping_neg()
                        } else {
                            self.mem.write_bytes(buf, cwd.as_bytes())?;
                            self.mem.write_u8(buf + n - 1, 0)?;
                            n
                        }
                    }
                    Err(e) => e.wrapping_neg(),
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(r))
    }

    // ---- the newcore file words ----------------------------------------

    /// `PROC openr` / `openrw` / `io_create` ( a n -- fd ).
    pub(crate) fn word_open(&mut self, host: &mut dyn Host, access: Access) -> R {
        let n = self.upop()?;
        let a = self.eax();
        let path = self.path_at(a, n)?;
        let fd = self.sys_open(host, &path, access);
        self.set_eax(fd);
        self.set_ior(fd)
    }

    /// `PROC io_close` ( fd -- ).
    pub(crate) fn word_close(&mut self, host: &mut dyn Host) -> R {
        let fd = self.upop()?;
        let r = self.sys_close(host, fd);
        self.set_ior(r)
    }

    /// `PROC io_read` ( a n fd -- n' ): bytes read, or `-errno`.
    pub(crate) fn word_read(&mut self, host: &mut dyn Host) -> R {
        let fd = self.upop()?;
        let n = self.upop()?;
        let a = self.eax();
        let r = self.sys_read(host, fd, a, n)?;
        self.set_eax(r);
        self.set_ior(r)
    }

    /// `PROC io_write` ( a n fd -- ).
    pub(crate) fn word_write(&mut self, host: &mut dyn Host) -> R {
        let fd = self.upop()?;
        let n = self.upop()?;
        let a = self.upop()?;
        let r = self.sys_write(host, fd, a, n)?;
        self.set_ior(r)
    }

    /// `(save)` ( a n -- ): write an image of the system to the file named
    /// by `a n` (see `image.rs`).  The original wrote a native executable;
    /// an image is its portable form, started with `reva --image` or by a
    /// page (`start(..., { image })`).  `ioerr` as for the file words.
    pub(crate) fn word_save(&mut self, host: &mut dyn Host) -> R {
        let n = self.upop()?;
        let a = self.upop()?;
        let path = self.path_at(a, n)?;
        let image = self.save_image();
        let r = ret(with_fs(host, |fs| {
            let fd = fs.open(&path, Access::Create)?;
            let w = fs.write(fd, &image);
            fs.close(fd)?;
            w.map(|_| 0)
        }));
        self.set_ior(r)
    }

    /// `PROC io_size` ( fd -- size ).  Like the original, `ioerr` is left
    /// alone; a bad descriptor reads as size 0.
    pub(crate) fn word_fsize(&mut self, host: &mut dyn Host) -> R {
        let fd = self.eax();
        let size =
            with_fs(host, |fs| fs.fstat(fd)).map_or(0, |s| s.size.min(u32::MAX as u64) as u32);
        self.set_eax(size);
        Ok(())
    }

    // ---- the virtual libc's directory functions ----------------------

    /// `DIR *opendir(const char *)`, or NULL (with `errno`).
    pub(crate) fn c_opendir(&mut self, host: &mut dyn Host, name: u32) -> R<u32> {
        let r = self.zpath(name).and_then(|p| {
            with_fs(host, |fs| {
                let names = fs.list_dir(&p)?;
                Ok((p, names))
            })
        });
        match r {
            Ok((path, names)) => {
                let mut all = vec![".".to_string(), "..".to_string()];
                all.extend(names);
                let s = DirStream {
                    path,
                    names: all,
                    next: 0,
                };
                let i = match self.dirs.iter().position(Option::is_none) {
                    Some(i) => {
                        self.dirs[i] = Some(s);
                        i
                    }
                    None => {
                        self.dirs.push(Some(s));
                        self.dirs.len() - 1
                    }
                };
                Ok(DIR_HANDLE_BASE + i as u32)
            }
            Err(e) => {
                self.mem.write_u32(st::ERRNO, e)?;
                Ok(0)
            }
        }
    }

    fn dir_stream(&mut self, h: u32) -> Option<&mut DirStream> {
        let i = h.checked_sub(DIR_HANDLE_BASE)? as usize;
        self.dirs.get_mut(i)?.as_mut()
    }

    /// `struct dirent *readdir(DIR *)` (glibc i386 layout: the name at
    /// offset 11), or NULL at the end.
    pub(crate) fn c_readdir(&mut self, host: &mut dyn Host, h: u32) -> R<u32> {
        let Some(d) = self.dir_stream(h) else {
            self.mem.write_u32(st::ERRNO, EBADF)?;
            return Ok(0);
        };
        let Some(name) = d.names.get(d.next).cloned() else {
            return Ok(0);
        };
        d.next += 1;
        let next = d.next as u32;
        let full = format!("{}/{name}", d.path);
        let is_dir =
            name == "." || name == ".." || with_fs(host, |fs| fs.stat(&full)).is_ok_and(|s| s.dir);
        let p = self.ffi_buffer(st::DIRENT_BUF, None, DIRENT_SIZE + 1)?;
        self.mem.fill(p, DIRENT_SIZE + 1, 0)?;
        self.mem.write_u32(p, 1)?; // d_ino
        self.mem.write_u32(p + 4, next)?; // d_off
        self.mem.write_u16(p + 8, DIRENT_SIZE as u16)?;
        self.mem.write_u8(p + 10, if is_dir { 4 } else { 8 })?; // DT_DIR / DT_REG
        let bytes = name.as_bytes();
        self.mem
            .write_bytes(p + 11, &bytes[..bytes.len().min(255)])?;
        Ok(p)
    }

    /// `int closedir(DIR *)`.
    pub(crate) fn c_closedir(&mut self, h: u32) -> R<u32> {
        let i = h.wrapping_sub(DIR_HANDLE_BASE) as usize;
        match self.dirs.get_mut(i) {
            Some(slot @ Some(_)) => {
                *slot = None;
                Ok(0)
            }
            _ => Ok(u32::MAX),
        }
    }
}
