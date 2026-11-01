//! # reva-rawterm — raw terminal mode, per platform
//!
//! `raw::Raw::enter()` puts stdin in raw mode until dropped (None if it
//! cannot), and `raw::width()` asks the terminal for its width (None if it
//! cannot say).  No crates: Unix uses `tcgetattr`/`cfmakeraw`/`tcsetattr`
//! from the C library through an opaque buffer, and `TIOCGWINSZ`; Windows
//! uses `SetConsoleMode` with virtual terminal input.

#[cfg(not(any(unix, windows)))]
pub mod other;
#[cfg(unix)]
pub mod unix;
#[cfg(unix)]
pub mod unix_width;
#[cfg(windows)]
pub mod windows;

/// Raw mode and the width on this platform.
#[cfg(not(any(unix, windows)))]
pub use other as raw;
/// Raw mode and the width on this platform.
#[cfg(unix)]
pub mod raw {
    pub use crate::unix::Raw;
    pub use crate::unix_width::width;
}
/// Raw mode and the width on this platform.
#[cfg(windows)]
pub use windows as raw;
