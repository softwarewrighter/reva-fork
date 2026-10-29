//! # reva-words-ffi — the portable OS layer: `(lib)`, `(func)`, `(call)`
//!
//! `reva.f` reaches the operating system in two ways, and both are kept so
//! that its OS personality (the `LIN [IF]` sections) runs unchanged:
//!
//! * **FFI** — `lib`/`func:`/`data:` resolve C functions lazily through
//!   `(lib)` (`dlopen`), `(func)` (`dlsym`) and `(call)` (cdecl call).  The
//!   port serves *virtual* libraries: libc (reva-words-libc) and those of
//!   reva-vlib, whose functions are gateway routines over `Host`.  Any
//!   other library, or a function not in [`libs::HOST_FUNCS`], resolves to
//!   0 — exactly what a failed `dlopen`/`dlsym` returned — so `reva.f`
//!   reports "could not load" with the function's name.
//! * **`syscall`** — Linux i386 system calls by number (reva-words-libc).
//!
//! **No native libraries, by design.**  Forth passes 32-bit VM addresses as
//! pointers, which native 64-bit (or wasm) code cannot use, so a generic
//! `dlopen` path cannot work safely.  More C libraries arrive the way libc
//! did: as *virtual libraries*, Rust bindings that marshal their arguments
//! (see docs/portability.md).  When a library is not served, the first
//! function looked up in it prints a one-line note naming the library
//! (`Host::diag`), so "could not load iconv_open" comes with the reason.
//!
//! `system` (Reva's `shell`) is served, but runs only where the host allows
//! it (`Host::system`): the CLI with `--allow-shell`, never the browser.
//!
//! [`os`] holds the gate handlers of this layer and of the file words.

pub mod call;
pub mod libs;
pub mod loader;
pub mod os;
