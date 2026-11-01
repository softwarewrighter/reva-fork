//! 32-bit stores and blocks: slices, copies, fills.

use crate::memory::Memory;
use reva_x86_base::fault::Fault;

impl Memory {
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
}
