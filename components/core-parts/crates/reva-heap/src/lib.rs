//! # reva-heap — the VM heap's allocator
//!
//! [`heap::Heap`]: first-fit allocation for `allocate`/`free`/`resize` and
//! `slurp` ([`alloc`]), carried through images ([`persist`]).

pub mod alloc;
pub mod heap;
pub mod persist;
