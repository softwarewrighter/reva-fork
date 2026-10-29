//! # reva-words — the gateway routines of the Reva core
//!
//! The `newcore.asm` routines in Rust ([`gates::Words`]), the OS layer
//! ([`ffi::Ffi`]: `(lib)`, `(func)`, `(call)`, `syscall`, the virtual libc)
//! and the file words ([`fileio::FileIo`]), as extension traits of
//! reva-core's `Machine`.  [`dispatch::dispatch`] is the function machines are built
//! with ([`dispatch::dispatch`]; docs/design.md 3.12).

pub mod dispatch;
pub mod ffi;
pub mod fileio;
pub mod gates;

// Paths the moved code uses (crate::x86, crate::host, ...), from the
// crates they live in.
pub use reva_core::{Gate, Machine, Next, st};
pub use reva_core::{canvas, gate, heap, image, machine, translate};
pub use reva_host::{charset, embedded, host, hostcall, version, vfs};
pub use reva_vlib::{highlight, iconv, sqlite, turtle};
pub use reva_x86 as x86;
pub use reva_x86::{fault, layout, mem};
