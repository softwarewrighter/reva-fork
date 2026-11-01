//! Immediates of an operand size, branch targets, and the decode fault.

use crate::cursor::Cursor;
use reva_x86_base::fault::Fault;
use reva_x86_types::operand::Size;

impl Cursor<'_> {
    /// An immediate of `size`.
    pub fn imm(&mut self, size: Size) -> Result<u32, Fault> {
        Ok(match size {
            Size::B => self.u8()? as u32,
            Size::W => self.u16()? as u32,
            Size::D => self.u32()?,
        })
    }

    /// Relative branch target: displacement is relative to the *end* of
    /// the instruction, which is the cursor position after reading it.
    pub fn rel8(&mut self) -> Result<u32, Fault> {
        let d = self.i8()?;
        Ok(self.pos.wrapping_add(d))
    }

    pub fn rel32(&mut self) -> Result<u32, Fault> {
        let d = self.u32()?;
        Ok(self.pos.wrapping_add(d))
    }

    /// The fault for an instruction the decoder does not know, with up to
    /// eight of its bytes.
    pub fn bad(&self) -> Fault {
        let bytes = (0..8)
            .map_while(|i| self.mem.read_u8(self.start.wrapping_add(i)).ok())
            .collect();
        Fault::BadOpcode {
            eip: self.start,
            bytes,
        }
    }
}
