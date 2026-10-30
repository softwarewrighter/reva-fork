//! # reva-vlib-turtle — the virtual library `turtle`
//!
//! Forth reaches the reva-canvas's `Canvas` the way it reaches libc: `" turtle" lib`
//! then `func:` per function, resolved by `(lib)`/`(func)` and called with
//! `(call)` (see reva-words-ffi and `lib/turtle`).  Arguments are integers
//! (pixels, degrees, `0xRRGGBB`); the turtle's own arithmetic is `f64`.
//!
//! Every function returns a value (C `int`); those that only act return 0,
//! so `vfunc:` drops it.

pub mod call;
pub mod names;
pub mod state;

pub use call::Turtle;
pub use names::{TURTLE_FUNCS, TURTLE_HANDLE, TURTLE_NAMES, lookup};
