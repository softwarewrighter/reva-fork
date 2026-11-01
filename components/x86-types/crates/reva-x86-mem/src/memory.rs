//! # VM memory
//!
//! One `Vec<u8>` addressed by `u32`.  All accesses are bounds-checked and the
//! null guard faults, so a wild pointer in Forth code produces a [`Fault`]
//! instead of undefined behaviour in the host.
//!
//! ## Self-modifying code
//!
//! Reva patches code it has already emitted (`then` back-patches jumps, `;`
//! turns a trailing `call` into a `jmp`, `does>` rewrites a word's class).
//! Anything that caches decoded or translated code must therefore learn about
//! writes to code it has seen.  Memory keeps a bitmap of *code pages*
//! (256 bytes each); the decoder marks a page when it decodes an instruction
//! there, and any write into a marked page clears the mark and queues the page
//! on the *dirty list* (see [`crate::pages`]).  Caches drain that list before
//! their next lookup.  Writes to plain data pages cost one bit test.

use reva_x86_base::fault::Fault;
use reva_x86_base::layout::NULL_GUARD;

pub const PAGE_SHIFT: u32 = 8;
pub const PAGE_SIZE: u32 = 1 << PAGE_SHIFT;

/// The VM's memory; its accesses are in [`crate::access`],
/// [`crate::bulk`] and [`crate::pages`].
pub struct Memory {
    pub(crate) bytes: Vec<u8>,
    pub(crate) code_pages: Vec<u64>,
    pub(crate) dirty: Vec<u32>,
}

impl Memory {
    pub fn new(size: u32) -> Memory {
        let pages = (size >> PAGE_SHIFT) as usize + 1;
        Memory {
            bytes: vec![0; size as usize],
            code_pages: vec![0; pages.div_ceil(64)],
            dirty: Vec::new(),
        }
    }

    pub fn size(&self) -> u32 {
        self.bytes.len() as u32
    }

    /// The byte range `addr..addr+len`, or the fault for touching it.
    #[inline]
    pub(crate) fn range(&self, addr: u32, len: u32) -> Result<std::ops::Range<usize>, Fault> {
        let end = addr as u64 + len as u64;
        if addr < NULL_GUARD || end > self.bytes.len() as u64 {
            return Err(Fault::BadAddress { addr, len });
        }
        Ok(addr as usize..end as usize)
    }

    /// Raw view of the whole memory (for images and translators).
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}
