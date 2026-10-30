//! # reva-core — the Reva machine's state
//!
//! The vx86 CPU and memory (reva-x86) with Reva's state around them: the
//! static variables, the stacks, the heap, the canvas, and the helpers
//! every gateway uses ([`stack`], [`cells`], [`vars`]).  The rest of the
//! machine is extension traits in crates above this one: reva-core-ffi
//! (`(call)` arguments), reva-core-inspect (the dictionary, state for
//! hosts), reva-core-run (the run loop), reva-core-boot (building
//! machines, images).  A machine dispatches each [`Gate`] through the
//! function it is built with (reva-vm wires reva-words'), so this crate
//! stays near the bottom of the stack (docs/design.md 3.12, 3.13).

pub mod cells;
pub mod machine;
pub mod stack;
pub mod vars;

// Paths the moved code uses (crate::x86, crate::host, ...), from the
// crates they live in.
pub use machine::{DirStream, Dispatch, Flow, Machine, Next, Status, st};
pub use reva_canvas::canvas;
pub use reva_gate::gate;
pub use reva_gate::gate as gates;
pub use reva_gate::gate::Gate;
pub use reva_heap::heap;
pub use reva_host::{charset, embedded, host, hostcall, version, vfs};
pub use reva_translate::translate;
pub use reva_x86 as x86;
pub use reva_x86::{fault, layout, mem};
