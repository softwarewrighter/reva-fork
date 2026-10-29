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
//! on the *dirty list*.  Caches drain that list before their next lookup.
//! Writes to plain data pages cost one bit test.

use crate::fault::Fault;
use crate::layout::NULL_GUARD;

pub const PAGE_SHIFT: u32 = 8;
pub const PAGE_SIZE: u32 = 1 << PAGE_SHIFT;

pub struct Memory {
    bytes: Vec<u8>,
    code_pages: Vec<u64>,
    dirty: Vec<u32>,
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

    #[inline]
    fn range(&self, addr: u32, len: u32) -> Result<std::ops::Range<usize>, Fault> {
        let end = addr as u64 + len as u64;
        if addr < NULL_GUARD || end > self.bytes.len() as u64 {
            return Err(Fault::BadAddress { addr, len });
        }
        Ok(addr as usize..end as usize)
    }

    #[inline]
    pub fn read_u8(&self, addr: u32) -> Result<u8, Fault> {
        let r = self.range(addr, 1)?;
        Ok(self.bytes[r.start])
    }

    #[inline]
    pub fn read_u16(&self, addr: u32) -> Result<u16, Fault> {
        let r = self.range(addr, 2)?;
        Ok(u16::from_le_bytes([
            self.bytes[r.start],
            self.bytes[r.start + 1],
        ]))
    }

    #[inline]
    pub fn read_u32(&self, addr: u32) -> Result<u32, Fault> {
        let r = self.range(addr, 4)?;
        let b = &self.bytes[r];
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    #[inline]
    pub fn write_u8(&mut self, addr: u32, v: u8) -> Result<(), Fault> {
        let r = self.range(addr, 1)?;
        self.note_write(addr, 1);
        self.bytes[r.start] = v;
        Ok(())
    }

    #[inline]
    pub fn write_u16(&mut self, addr: u32, v: u16) -> Result<(), Fault> {
        let r = self.range(addr, 2)?;
        self.note_write(addr, 2);
        self.bytes[r].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    #[inline]
    pub fn write_u32(&mut self, addr: u32, v: u32) -> Result<(), Fault> {
        let r = self.range(addr, 4)?;
        self.note_write(addr, 4);
        self.bytes[r].copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn slice(&self, addr: u32, len: u32) -> Result<&[u8], Fault> {
        let r = self.range(addr, len)?;
        Ok(&self.bytes[r])
    }

    pub fn write_bytes(&mut self, addr: u32, data: &[u8]) -> Result<(), Fault> {
        let r = self.range(addr, data.len() as u32)?;
        self.note_write(addr, data.len() as u32);
        self.bytes[r].copy_from_slice(data);
        Ok(())
    }

    /// Byte-at-a-time forward copy, exactly like Reva's `_move` (so an
    /// overlapping copy to a higher address smears, as it always did).
    pub fn move_forward(&mut self, src: u32, dst: u32, len: u32) -> Result<(), Fault> {
        if len == 0 {
            return Ok(());
        }
        self.range(src, len)?;
        let r = self.range(dst, len)?;
        self.note_write(dst, len);
        let (s, d) = (src as usize, r.start);
        if d <= s || d >= s + len as usize {
            self.bytes.copy_within(s..s + len as usize, d);
        } else {
            for i in 0..len as usize {
                self.bytes[d + i] = self.bytes[s + i];
            }
        }
        Ok(())
    }

    pub fn fill(&mut self, addr: u32, len: u32, v: u8) -> Result<(), Fault> {
        let r = self.range(addr, len)?;
        self.note_write(addr, len);
        self.bytes[r].fill(v);
        Ok(())
    }

    /// Raw view of the whole memory (for images and translators).
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    // ---- code-page tracking -------------------------------------------

    #[inline]
    pub fn mark_code(&mut self, addr: u32) {
        let p = (addr >> PAGE_SHIFT) as usize;
        if let Some(w) = self.code_pages.get_mut(p / 64) {
            *w |= 1 << (p % 64);
        }
    }

    #[inline]
    fn note_write(&mut self, addr: u32, len: u32) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_guard_and_bounds() {
        let mut m = Memory::new(0x2000);
        assert!(m.read_u32(0).is_err());
        assert!(m.read_u32(0x1ffd).is_err());
        m.write_u32(0x1ffc, 0xdeadbeef).unwrap();
        assert_eq!(m.read_u32(0x1ffc).unwrap(), 0xdeadbeef);
        assert_eq!(m.read_u8(0x1fff).unwrap(), 0xde);
    }

    #[test]
    fn writes_to_code_pages_are_reported_once() {
        let mut m = Memory::new(0x4000);
        m.write_u8(0x1100, 1).unwrap();
        assert!(!m.has_dirty());
        m.mark_code(0x1100);
        m.write_u32(0x11fe, 7).unwrap(); // spans pages 0x1100 and 0x1200
        assert_eq!(m.take_dirty(), vec![0x1100]);
        m.write_u8(0x1100, 2).unwrap();
        assert!(!m.has_dirty());
    }

    #[test]
    fn overlapping_forward_move_smears_like_reva() {
        let mut m = Memory::new(0x2000);
        m.write_bytes(0x1000, b"ab....").unwrap();
        m.move_forward(0x1000, 0x1002, 4).unwrap();
        assert_eq!(m.slice(0x1000, 6).unwrap(), b"ababab");
    }
}
