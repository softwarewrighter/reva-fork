//! The virtual libc's directory functions: `opendir`, `readdir`,
//! `closedir`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::vfs::errno::EBADF;
use reva_core::{DirStream, st};
use reva_core_ffi::regs::MachineRegs;
use reva_words_sys::fs::with_fs;
use reva_words_sys::paths::Paths;

type R<T = ()> = Result<T, Fault>;

/// `opendir` handles: opaque, nonzero, never a valid address.
const DIR_HANDLE_BASE: u32 = 0x4449_0000; // "DI"
const DIRENT_SIZE: u32 = 11 + 256;

reva_macros::machine_trait! {
    /// Directory streams over the host's file system.
    pub trait Dirs: Paths {
        /// `DIR *opendir(const char *)`, or NULL (with `errno`).
        fn c_opendir(&mut self, host: &mut dyn Host, name: u32) -> R<u32> {
            let r = self
                .zpath(name)
                .and_then(|p| with_fs(host, |fs| Ok((fs.list_dir(&p)?, p))));
            match r {
                Ok((names, path)) => Ok(self.store_dir(path, names)),
                Err(e) => {
                    self.mem.write_u32(st::ERRNO, e)?;
                    Ok(0)
                }
            }
        }

        /// Keep a stream of `.`, `..` and `names` in a free slot: its handle.
        fn store_dir(&mut self, path: String, names: Vec<String>) -> u32 {
            let mut all = vec![".".to_string(), "..".to_string()];
            all.extend(names);
            let s = Some(DirStream { path, names: all, next: 0 });
            let i = match self.dirs.iter().position(Option::is_none) {
                Some(i) => {
                    self.dirs[i] = s;
                    i
                }
                None => {
                    self.dirs.push(s);
                    self.dirs.len() - 1
                }
            };
            DIR_HANDLE_BASE + i as u32
        }

        /// The open stream behind handle `h`.
        fn dir_stream(&mut self, h: u32) -> Option<&mut DirStream> {
            let i = h.checked_sub(DIR_HANDLE_BASE)? as usize;
            self.dirs.get_mut(i)?.as_mut()
        }

        /// `struct dirent *readdir(DIR *)` (glibc i386 layout: the name at
        /// offset 11), or NULL at the end.
        fn c_readdir(&mut self, host: &mut dyn Host, h: u32) -> R<u32> {
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
            self.mem.write_bytes(p + 11, &bytes[..bytes.len().min(255)])?;
            Ok(p)
        }

        /// `int closedir(DIR *)`.
        fn c_closedir(&mut self, h: u32) -> R<u32> {
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
}
