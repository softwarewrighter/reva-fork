//! # The portable OS layer: `(lib)`, `(func)`, `(call)`, `syscall`
//!
//! `reva.f` reaches the operating system in two ways, and both are kept so
//! that its OS personality (the `LIN [IF]` sections) runs unchanged:
//!
//! * **FFI** — `lib`/`func:`/`data:` resolve C functions lazily through
//!   `(lib)` (`dlopen`), `(func)` (`dlsym`) and `(call)` (cdecl call).  The
//!   port serves one *virtual* library, libc, whose functions are gateway
//!   routines implemented here over [`Host`].  Any other library, or a
//!   function not in [`HOST_FUNCS`], resolves to 0 — exactly what a failed
//!   `dlopen`/`dlsym` returned — so `reva.f` reports "could not load" with
//!   the function's name.
//! * **`syscall`** — Linux i386 system calls by number.  The few that
//!   `reva.f` issues outside file I/O (time, sleep, terminal ioctls, pid)
//!   are mapped to [`Host`]; the rest return `-ENOSYS`, as a kernel without
//!   them would.
//!
//! Nothing here touches the real machine directly: the VM stays a pure
//! function of its inputs, and a host decides what "the environment" and
//! "the clock" are (the CLI: the process; the browser: nothing, for now).
//!
//! **No native libraries, by design.**  Forth passes 32-bit VM addresses as
//! pointers, which native 64-bit (or wasm) code cannot use, so a generic
//! `dlopen` path cannot work safely.  More C libraries arrive the way libc
//! did: as *virtual libraries*, Rust bindings that marshal their arguments
//! (see docs/portability.md).  When a library is not served, the first
//! function looked up in it prints a one-line note naming the library
//! ([`Host::diag`]), so "could not load iconv_open" comes with the reason.
//!
//! `system` (Reva's `shell`) is served, but runs only where the host allows
//! it ([`Host::system`]): the CLI with `--allow-shell`, never the browser.

use crate::fault::Fault;
use crate::gates::Gate;
use crate::host::Host;
use crate::machine::{Machine, Next, st};
use crate::x86::{EAX, ESI};

type R<T = ()> = Result<T, Fault>;

/// Handle `(lib)` returns for the virtual libc.  Nonzero, and not a valid
/// address, so it cannot be mistaken for a pointer.
pub const LIBC_HANDLE: u32 = 0x4c49_4243; // "LIBC"

/// Library names accepted as libc (the names `reva.f` uses per OS).
const LIBC_NAMES: &[&str] = &[
    "libc.so.6",
    "libc.so",
    "libc.dylib",
    "libc",
    "msvcrt",
    "msvcrt.dll",
];

/// C functions the virtual libc provides, by name, and their gateways.
pub const HOST_FUNCS: &[(&str, Gate)] = &[
    ("getenv", Gate::HostGetenv),
    ("putenv", Gate::HostPutenv),
    ("localtime", Gate::HostLocaltime),
    ("gmtime", Gate::HostLocaltime),
    ("opendir", Gate::HostOpendir),
    ("readdir", Gate::HostReaddir),
    ("closedir", Gate::HostClosedir),
    ("system", Gate::HostSystem),
];

/// `errno` is data, not code: `data: errno` reads through this cell.
const DATA_SYMBOLS: &[(&str, u32)] = &[("errno", st::ERRNO)];

// Linux i386 numbers used by `reva.f`.
const SYS_TIME: u32 = 13;
const SYS_GETPID: u32 = 20;
const SYS_IOCTL: u32 = 54;
const SYS_GETTIMEOFDAY: u32 = 78;
const SYS_NANOSLEEP: u32 = 162;
const ENOSYS: u32 = 38;
const ENOTTY: u32 = 25;

impl Machine {
    fn ffi_string(&self, a: u32, n: u32) -> R<String> {
        Ok(String::from_utf8_lossy(self.mem.slice(a, n)?).into_owned())
    }

    fn ffi_zstring(&self, a: u32) -> R<String> {
        let mut n = 0;
        while self.mem.read_u8(a + n)? != 0 {
            n += 1;
        }
        self.ffi_string(a, n)
    }

    fn reg(&self, r: u8) -> u32 {
        self.cpu.r[r as usize]
    }

    fn set_reg(&mut self, r: u8, v: u32) {
        self.cpu.r[r as usize] = v;
    }

    /// Pop the top data-stack cell (EAX) — the `upop` macro.
    fn ffi_pop(&mut self) -> R<u32> {
        let v = self.reg(EAX);
        let sp = self.reg(ESI);
        self.set_reg(EAX, self.mem.read_u32(sp)?);
        self.set_reg(ESI, sp.wrapping_add(4));
        Ok(v)
    }

    /// `PROC _loadlib` ( a n -- handle ).
    pub(crate) fn ffi_loadlib(&mut self) -> R {
        let n = self.ffi_pop()?;
        let a = self.reg(EAX);
        let name = self.ffi_string(a, n)?;
        let is = |names: &[&str]| names.iter().any(|l| l.eq_ignore_ascii_case(&name));
        let handle = if is(LIBC_NAMES) {
            LIBC_HANDLE
        } else if is(crate::turtle::TURTLE_NAMES) {
            crate::turtle::TURTLE_HANDLE
        } else if is(crate::highlight::HL_NAMES) {
            crate::highlight::HL_HANDLE
        } else if is(crate::sqlite::SQL_NAMES) {
            crate::sqlite::SQL_HANDLE
        } else if is(crate::iconv::ICONV_NAMES) {
            crate::iconv::ICONV_HANDLE
        } else {
            0
        };
        self.missing_lib = (handle == 0).then_some(name);
        self.set_reg(EAX, handle);
        Ok(())
    }

    /// `PROC _unloadlib` ( handle -- ): nothing to release.
    pub(crate) fn ffi_unloadlib(&mut self) -> R {
        self.ffi_pop().map(drop)
    }

    /// `PROC _osfunc` ( a n handle -- fn ): the address of a host function,
    /// or of a data symbol, or 0.  A lookup in a library `(lib)` could not
    /// serve (handle 0) notes that library once per session: `reva.f`
    /// resolves lazily, so the library's own load is silent until then.
    pub(crate) fn ffi_func(&mut self, host: &mut dyn Host) -> R {
        let handle = self.ffi_pop()?;
        let n = self.ffi_pop()?;
        let a = self.reg(EAX);
        let name = self.ffi_string(a, n)?;
        let name = name.trim_end_matches('\0');
        let mut addr = 0;
        if handle == LIBC_HANDLE {
            if let Some(&(_, g)) = HOST_FUNCS.iter().find(|(f, _)| *f == name) {
                addr = g.addr();
            } else if let Some(&(_, cell)) = DATA_SYMBOLS.iter().find(|(d, _)| *d == name) {
                addr = cell;
            }
        }
        if handle == crate::turtle::TURTLE_HANDLE {
            addr = crate::turtle::lookup(name).map_or(0, Gate::addr);
        }
        if handle == crate::highlight::HL_HANDLE {
            addr = crate::highlight::lookup(name).map_or(0, Gate::addr);
        }
        if handle == crate::sqlite::SQL_HANDLE {
            addr = crate::sqlite::lookup(name).map_or(0, Gate::addr);
        }
        if handle == crate::iconv::ICONV_HANDLE {
            addr = crate::iconv::lookup(name).unwrap_or(0);
        }
        if handle == 0
            && let Some(lib) = self.missing_lib.take()
            && !self.reported_libs.contains(&lib)
        {
            host.diag(&format!(
                "note: native library {lib} is not available in the portable core \
                         (see {})\n",
                crate::fault::PORTABILITY_DOC
            ));
            self.reported_libs.push(lib);
        }
        self.set_reg(EAX, addr);
        Ok(())
    }

    /// `PROC _call` ( x1 .. xn fn n -- result ): a cdecl call.  Only host
    /// functions can be called; the first C argument is the deepest of the
    /// `n` cells, as when the original pushed them onto the C stack.
    pub(crate) fn ffi_call(&mut self, host: &mut dyn Host) -> R<Next> {
        // Host calls read their arguments in place and pop them only once
        // the answer is in, so a call that waits can be retried.
        let n = self.reg(EAX);
        let f = self.mem.read_u32(self.reg(ESI))?;
        if let Some(next) = Gate::from_addr(f).and_then(|g| self.sqlite_call(host, g, n)) {
            return next;
        }
        let n = self.ffi_pop()?;
        let f = self.ffi_pop()?;
        if n > 16 {
            return Err(Fault::NotImplemented("(call) with more than 16 parameters"));
        }
        let mut args = vec![0u32; n as usize];
        for i in (0..n as usize).rev() {
            args[i] = self.ffi_pop()?;
        }
        let arg0 = args.first().copied().unwrap_or(0);
        let result = match Gate::from_addr(f) {
            Some(Gate::HostGetenv) => self.c_getenv(host, arg0)?,
            Some(Gate::HostPutenv) => self.c_putenv(host, arg0)?,
            Some(Gate::HostLocaltime) => self.c_localtime(host, arg0)?,
            Some(Gate::HostOpendir) => self.c_opendir(host, arg0)?,
            Some(Gate::HostReaddir) => self.c_readdir(host, arg0)?,
            Some(Gate::HostClosedir) => self.c_closedir(arg0)?,
            Some(Gate::HostSystem) => self.c_system(host, arg0)?,
            Some(g) => match self.iconv_call(g, &args)? {
                Some(r) => r,
                None => match self.highlight_call(host, g, &args)? {
                    Some(r) => r,
                    None => match self.turtle_call(host, g, &args)? {
                        Some(r) => r,
                        None => return Err(Fault::NativeCall { addr: f }),
                    },
                },
            },
            None => return Err(Fault::NativeCall { addr: f }),
        };
        self.ffi_push(result)?;
        Ok(Next::Ret)
    }

    /// End a `(call)` of `n` cells read in place: drop them, the function
    /// address and the count, and push `result`.
    pub(crate) fn ffi_finish(&mut self, n: u32, result: u32) -> R {
        for _ in 0..n + 2 {
            self.ffi_pop()?;
        }
        self.ffi_push(result)
    }

    /// upsh: `v` becomes TOS.
    fn ffi_push(&mut self, v: u32) -> R {
        let sp = self.reg(ESI).wrapping_sub(4);
        self.mem.write_u32(sp, self.reg(EAX))?;
        self.set_reg(ESI, sp);
        self.set_reg(EAX, v);
        Ok(())
    }

    /// A heap buffer of at least `len` bytes, kept in statics cells
    /// `buf`/`cap` (so it survives in saved images).
    pub(crate) fn ffi_buffer(&mut self, buf: u32, cap: Option<u32>, len: u32) -> R<u32> {
        let have = self.mem.read_u32(buf)?;
        let have_len = match cap {
            Some(c) => self.mem.read_u32(c)?,
            None => len,
        };
        if have != 0 && have_len >= len {
            return Ok(have);
        }
        if have != 0 {
            self.heap.free(have);
        }
        let size = len.max(64);
        let p = self.heap.alloc(size).ok_or(Fault::OutOfSpace("heap"))?;
        self.mem.write_u32(buf, p)?;
        if let Some(c) = cap {
            self.mem.write_u32(c, size)?;
        }
        Ok(p)
    }

    /// `char *getenv(const char *)`: the value, zero-terminated, in a
    /// buffer reused by the next call (as C allows), or NULL.
    fn c_getenv(&mut self, host: &mut dyn Host, name: u32) -> R<u32> {
        let name = self.ffi_zstring(name)?;
        let Some(value) = host.getenv(&name) else {
            return Ok(0);
        };
        let bytes = value.as_bytes();
        let p = self.ffi_buffer(st::ENV_BUF, Some(st::ENV_LEN), bytes.len() as u32 + 1)?;
        self.mem.write_bytes(p, bytes)?;
        self.mem.write_u8(p + bytes.len() as u32, 0)?;
        Ok(p)
    }

    /// `int system(const char *)`: the host runs the command if it allows
    /// that, else the call faults with how to allow it.  `system(NULL)`
    /// asks whether a shell exists: 0 here, as nothing is promised.
    fn c_system(&mut self, host: &mut dyn Host, cmd: u32) -> R<u32> {
        if cmd == 0 {
            return Ok(0);
        }
        let cmd = self.ffi_zstring(cmd)?;
        match host.system(&cmd) {
            Some(status) => Ok(status as u32),
            None => Err(Fault::NotPermitted {
                what: "shell (running an OS command)",
                hint: "the CLI allows it with --allow-shell; the browser never does",
            }),
        }
    }

    /// `int putenv(char *)` with `name=value`.
    fn c_putenv(&mut self, host: &mut dyn Host, s: u32) -> R<u32> {
        let s = self.ffi_zstring(s)?;
        match s.split_once('=') {
            Some((k, v)) => {
                host.setenv(k, v);
                Ok(0)
            }
            None => Ok(u32::MAX),
        }
    }

    /// `struct tm *localtime(const time_t *)`.  The port has no time-zone
    /// database, so local time is UTC (documented in docs/gotchas.md).
    fn c_localtime(&mut self, _host: &mut dyn Host, tp: u32) -> R<u32> {
        let t = self.mem.read_u32(tp)? as i32 as i64;
        let tm = tm_from_unix(t);
        let p = self.ffi_buffer(st::TM_BUF, None, 11 * 4)?;
        for (i, v) in tm.iter().enumerate() {
            self.mem.write_u32(p + 4 * i as u32, *v as u32)?;
        }
        self.mem.write_u32(p + 36, 0)?; // tm_gmtoff
        self.mem.write_u32(p + 40, 0)?; // tm_zone
        Ok(p)
    }

    /// `os_syscall` ( p1 .. pN N num -- result ): the topmost argument is
    /// the first (EBX), as in the original register loading.
    pub(crate) fn ffi_syscall(&mut self, host: &mut dyn Host) -> R {
        let num = self.reg(EAX);
        let sp = self.reg(ESI);
        let count = self.mem.read_u32(sp)?;
        if count > 6 {
            return Ok(());
        }
        let mut a = [0u32; 6];
        for (i, slot) in a.iter_mut().enumerate().take(count as usize) {
            *slot = self.mem.read_u32(sp + 4 + 4 * i as u32)?;
        }
        let file = self.sys_file(host, num, &a)?;
        let result = match num {
            _ if file.is_some() => file.unwrap_or(0),
            SYS_TIME => {
                let secs = host.now().map_or(0, |(s, _)| s as u32);
                if a[0] != 0 {
                    self.mem.write_u32(a[0], secs)?;
                }
                secs
            }
            SYS_GETTIMEOFDAY => {
                let (s, us) = host.now().unwrap_or((0, 0));
                if a[0] != 0 {
                    self.mem.write_u32(a[0], s as u32)?;
                    self.mem.write_u32(a[0] + 4, us)?;
                }
                0
            }
            SYS_NANOSLEEP => {
                if a[0] != 0 {
                    let s = self.mem.read_u32(a[0])?;
                    let ns = self.mem.read_u32(a[0] + 4)?;
                    host.sleep_ms(s.saturating_mul(1000).saturating_add(ns / 1_000_000));
                }
                0
            }
            SYS_GETPID => 1,
            SYS_IOCTL => ENOTTY.wrapping_neg(),
            _ => ENOSYS.wrapping_neg(),
        };
        self.set_reg(ESI, sp + 4 + 4 * count);
        self.set_reg(EAX, result);
        Ok(())
    }
}

/// `struct tm` fields (sec, min, hour, mday, mon 0-11, year-1900, wday,
/// yday, isdst) for a Unix time, in UTC.
pub fn tm_from_unix(t: i64) -> [i32; 9] {
    let days = t.div_euclid(86_400);
    let secs = t.rem_euclid(86_400);
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let mday = doy - (153 * mp + 2) / 5 + 1;
    let mon = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(mon <= 2);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    const CUM: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let yday = CUM[(mon - 1) as usize] + mday - 1 + i64::from(leap && mon > 2);
    let wday = (days + 4).rem_euclid(7);
    [
        (secs % 60) as i32,
        (secs / 60 % 60) as i32,
        (secs / 3600) as i32,
        mday as i32,
        (mon - 1) as i32,
        (year - 1900) as i32,
        wday as i32,
        yday as i32,
        0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tm_known_dates() {
        assert_eq!(tm_from_unix(0), [0, 0, 0, 1, 0, 70, 4, 0, 0]);
        // 2000-02-29 12:34:56 UTC, a Tuesday, day 59 of a leap year
        assert_eq!(
            tm_from_unix(951_827_696),
            [56, 34, 12, 29, 1, 100, 2, 59, 0]
        );
        // 2026-10-25 00:00:00 UTC, a Sunday
        assert_eq!(
            tm_from_unix(1_792_886_400),
            [0, 0, 0, 25, 9, 126, 0, 297, 0]
        );
    }
}
