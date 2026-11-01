//! Bytes, words and the 16-bit stores.

use crate::memory::Memory;
use reva_x86_base::fault::Fault;

impl Memory {
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
}
