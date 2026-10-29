//! # reva-core — the Reva machine
//!
//! The vx86 CPU and memory (reva-x86) with Reva's state around them: the
//! static variables, the stacks, the heap, saved images, and the run loop
//! that steps instructions and enters gateways.  The gateway routines are
//! not here: a machine dispatches each [`gate::Gate`] through the function
//! it is built with (reva-vm wires reva-words'), so this crate stays the
//! bottom of the stack (docs/design.md 3.12).

pub mod canvas;
pub mod gate;
pub mod heap;
pub mod image;
pub mod machine;
pub mod translate;

// Paths the moved code uses (crate::x86, crate::host, ...), from the
// crates they live in.
pub use gate as gates;
pub use gate::Gate;
pub use machine::{DirStream, Dispatch, Machine, Next, Status, st};
pub use reva_host::{charset, embedded, host, hostcall, version, vfs};
pub use reva_x86 as x86;
pub use reva_x86::{fault, layout, mem};
