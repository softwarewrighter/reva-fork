//! # reva-vm — a portable core for Reva Forth
//!
//! Reva's original core is ~2,000 lines of 32-bit x86 assembly, and the
//! Forth system built on it (`src/reva.f`) compiles directly to x86 machine
//! code and is partly *written* in machine code.  This crate keeps both:
//!
//! * [`x86`] executes the small IA-32 subset that Reva's compiler and its
//!   hand-written primitives use, over Reva's own register conventions and a
//!   flat 32-bit memory ([`layout`], [`mem`]) — the same model as `wasm32`;
//! * [`gates`] re-implements the `newcore.asm` routines in Rust, reachable
//!   at fixed addresses so compiled code calls them exactly as before;
//! * [`machine`] ties them together with a dispatcher that can pause for
//!   input after any step, so the same core runs under a blocking CLI and a
//!   browser event loop;
//! * [`translate`] defines the hooks for native (x86-64, ARM64) and
//!   WebAssembly translators — the path to native speed.
//!
//! See `docs/design.md` for the rationale, the trade-offs, and the roadmap.
//!
//! ```
//! use reva_vm::prelude::*;
//! use reva_vm::{BufferHost, Machine, Status};
//! let mut vm = Machine::new(&["reva", reva_vm::embedded::BOOT_PATH]);
//! let mut host = BufferHost::new();
//! assert_eq!(vm.run_until_blocked(&mut host), Status::NeedInput);
//! host.take_output(); // boot progress marks, banner, first prompt
//! vm.quiet_prompt();   // no `ok>` before each line
//! vm.feed(b": sq dup * ; 7 sq .\n");
//! vm.run_until_blocked(&mut host);
//! assert_eq!(host.take_output(), "49 ");
//! ```

pub mod machine;
pub mod prelude;
pub mod start;

// Every module of the one-crate core, from the crate it lives in now, under
// its old path (docs/design.md 3.12).
pub use reva_core::gate as gates;
pub use reva_core::{canvas, heap, translate};
pub use reva_core_boot::image;
pub use reva_host::{charset, embedded, host, hostcall, version, vfs};
pub use reva_vlib::{highlight, iconv, sqlite, turtle};
pub use reva_words::ffi;
pub use reva_x86 as x86;
pub use reva_x86::{fault, layout, mem};

pub use fault::Fault;
pub use host::{BufferHost, Host};
pub use image::ImageError;
pub use machine::{Machine, Status, st};
pub use start::Start;
