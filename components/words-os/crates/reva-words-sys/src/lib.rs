//! # reva-words-sys — file system calls over the host's file system
//!
//! Linux i386 file system calls with the kernel's register order and
//! result convention (the value, or `-errno`), over [`Host::fs`].  File
//! descriptors 0-2 stay the console: writing to 1 or 2 is output,
//! closing them does nothing, and reading 0 fails with `EBADF` (input
//! reaches Forth through `key`, which can pause the VM; `read` cannot).
//!
//! [`Host::fs`]: reva_core::host::Host::fs

pub mod fs;
pub mod paths;
pub mod syscalls;
pub mod sysio;
