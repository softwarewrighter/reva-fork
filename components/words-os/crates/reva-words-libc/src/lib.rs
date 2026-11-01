//! # reva-words-libc — the virtual libc and `syscall`
//!
//! The C functions the port serves as the library libc (`getenv`,
//! `putenv`, `system`, `localtime`; the directory functions are in
//! reva-words-files), and `os_syscall`: Linux i386 system calls by number.
//! The few that `reva.f` issues outside file I/O (time, sleep, terminal
//! ioctls, pid) are mapped to [`Host`]; the rest return `-ENOSYS`, as a
//! kernel without them would.  A host decides what "the environment"
//! and "the clock" are (the CLI: the process; the browser: nothing).
//!
//! [`Host`]: reva_core::host::Host

pub mod clock;
pub mod env;
pub mod syscall;
pub mod time;
