//! A small first-fit allocator for `allocate`/`free`/`resize` and `slurp`.
//!
//! Book-keeping lives on the Rust side (not in VM memory) so a wild store
//! from Forth cannot corrupt the allocator; [`Heap::save`]/[`Heap::load`]
//! carry it through images.

use std::collections::BTreeMap;

/// The allocator; [`crate::alloc`] and [`crate::persist`] hold its
/// operations.
pub struct Heap {
    pub(crate) base: u32,
    pub(crate) limit: u32,
    pub(crate) top: u32,
    pub(crate) used: BTreeMap<u32, u32>,
    pub(crate) free: BTreeMap<u32, u32>,
}

pub(crate) const ALIGN: u32 = 8;

impl Heap {
    pub fn new(base: u32, limit: u32) -> Heap {
        Heap {
            base,
            limit,
            top: base,
            used: BTreeMap::new(),
            free: BTreeMap::new(),
        }
    }

    /// Highest address handed out so far (end of the used heap).
    pub fn top(&self) -> u32 {
        self.top
    }

    pub fn size_of(&self, addr: u32) -> Option<u32> {
        self.used.get(&addr).copied()
    }
}
