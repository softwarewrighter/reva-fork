//! Code-page tracking: which writes touched decoded code.

use crate::memory::{Memory, PAGE_SHIFT};

impl Memory {
    /// The page holding `addr` has decoded code.
    #[inline]
    pub fn mark_code(&mut self, addr: u32) {
        let p = (addr >> PAGE_SHIFT) as usize;
        if let Some(w) = self.code_pages.get_mut(p / 64) {
            *w |= 1 << (p % 64);
        }
    }

    /// A write of `len` bytes at `addr`: queue the code pages it touches.
    #[inline]
    pub(crate) fn note_write(&mut self, addr: u32, len: u32) {
        let first = addr >> PAGE_SHIFT;
        let last = (addr + len - 1) >> PAGE_SHIFT;
        for p in first..=last {
            let (w, bit) = ((p / 64) as usize, 1u64 << (p % 64));
            if self.code_pages[w] & bit != 0 {
                self.code_pages[w] &= !bit;
                self.dirty.push(p << PAGE_SHIFT);
            }
        }
    }

    /// True when code pages were written since the last [`Self::take_dirty`].
    #[inline]
    pub fn has_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// Start addresses of code pages written since the last call.
    pub fn take_dirty(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.dirty)
    }
}
