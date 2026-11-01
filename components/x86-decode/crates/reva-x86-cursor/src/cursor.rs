//! The cursor and its fixed-size reads.

use reva_x86_base::fault::Fault;
use reva_x86_mem::memory::Memory;

/// A position in the instruction being decoded.
pub struct Cursor<'a> {
    pub mem: &'a Memory,
    /// The instruction's first byte (prefixes included).
    pub start: u32,
    /// The next byte to read.
    pub pos: u32,
}

impl<'a> Cursor<'a> {
    /// A cursor at `eip`.
    pub fn new(mem: &'a Memory, eip: u32) -> Cursor<'a> {
        Cursor {
            mem,
            start: eip,
            pos: eip,
        }
    }

    pub fn u8(&mut self) -> Result<u8, Fault> {
        let b = self.mem.read_u8(self.pos)?;
        self.pos = self.pos.wrapping_add(1);
        Ok(b)
    }

    /// A sign-extended byte.
    pub fn i8(&mut self) -> Result<u32, Fault> {
        Ok(self.u8()? as i8 as i32 as u32)
    }

    pub fn u16(&mut self) -> Result<u16, Fault> {
        let v = self.mem.read_u16(self.pos)?;
        self.pos = self.pos.wrapping_add(2);
        Ok(v)
    }

    pub fn u32(&mut self) -> Result<u32, Fault> {
        let v = self.mem.read_u32(self.pos)?;
        self.pos = self.pos.wrapping_add(4);
        Ok(v)
    }
}

/// The prefixes before an opcode: operand size (`66`) and `rep` (`F2`/`F3`).
#[derive(Clone, Copy, Debug)]
pub struct Prefixes {
    pub osz: reva_x86_types::operand::Size,
    pub rep: reva_x86_types::op::Rep,
}

/// A decoder for the opcodes starting with byte `b`, the cursor after it.
pub type Decoder = fn(&mut Cursor, u8, Prefixes) -> Result<reva_x86_types::op::Op, Fault>;
