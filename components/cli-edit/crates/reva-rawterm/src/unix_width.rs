//! The terminal's width on Unix: `TIOCGWINSZ` (`None` where it is not
//! known or fails; the caller falls back to `$COLUMNS`).

use std::os::raw::{c_int, c_ulong};

unsafe extern "C" {
    fn ioctl(fd: c_int, req: c_ulong, ...) -> c_int;
}

// Linux: 0x5413, except on MIPS, PowerPC and SPARC, which use the BSD
// encoding.  Should the ioctl fail, $COLUMNS (or 80) is used.
#[cfg(any(target_os = "linux", target_os = "android"))]
const TIOCGWINSZ: c_ulong = if cfg!(any(
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "powerpc",
    target_arch = "powerpc64",
    target_arch = "sparc64"
)) {
    0x4008_7468
} else {
    0x5413
};
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
const TIOCGWINSZ: c_ulong = 0x4008_7468;

/// Terminal columns.
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub fn width() -> Option<usize> {
    let mut ws = [0u16; 4]; // rows, cols, xpixel, ypixel
    // SAFETY: TIOCGWINSZ fills a struct winsize (four u16s).
    let ok = unsafe { ioctl(1, TIOCGWINSZ, ws.as_mut_ptr()) } == 0;
    (ok && ws[1] > 0).then_some(ws[1] as usize)
}

/// Terminal columns: `$COLUMNS` (no TIOCGWINSZ known here).
#[cfg(not(any(
    target_os = "linux",
    target_os = "android",
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
)))]
pub fn width() -> Option<usize> {
    crate::reader::columns()
}
