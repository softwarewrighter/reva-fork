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
//! See `rust/DESIGN.md` for the rationale, the trade-offs, and the roadmap.
//!
//! ```
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

pub mod canvas;
pub mod embedded;
pub mod fault;
pub mod ffi;
mod fileio;
pub mod gates;
pub mod heap;
pub mod host;
pub mod image;
pub mod layout;
pub mod machine;
pub mod mem;
pub mod start;
pub mod translate;
pub mod turtle;
pub mod version;
pub mod vfs;
pub mod x86;

pub use fault::Fault;
pub use host::{BufferHost, Host};
pub use image::ImageError;
pub use machine::{st, Machine, Status};
pub use start::Start;
