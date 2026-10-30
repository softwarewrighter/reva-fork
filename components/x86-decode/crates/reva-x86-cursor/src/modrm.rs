//! ModRM, SIB and displacements.

use crate::cursor::Cursor;
use reva_x86_base::fault::Fault;
use reva_x86_types::operand::{MemRef, Opd};

impl Cursor<'_> {
    /// ModRM (+SIB, +displacement).  Returns the `reg` field and the r/m
    /// operand.
    pub fn modrm(&mut self) -> Result<(u8, Opd), Fault> {
        let m = self.u8()?;
        let (md, reg, rm) = (m >> 6, (m >> 3) & 7, m & 7);
        if md == 3 {
            return Ok((reg, Opd::Reg(rm)));
        }
        let mut mr = self.mem_base(md, rm)?;
        match md {
            1 => mr.disp = mr.disp.wrapping_add(self.i8()?),
            2 => mr.disp = mr.disp.wrapping_add(self.u32()?),
            _ => {}
        }
        Ok((reg, Opd::Mem(mr)))
    }

    /// The base and index of a memory operand (SIB when `rm` is 4).
    fn mem_base(&mut self, md: u8, rm: u8) -> Result<MemRef, Fault> {
        let mut mr = MemRef {
            base: None,
            index: None,
            scale: 1,
            disp: 0,
        };
        if rm == 4 {
            self.sib(md, &mut mr)?;
        } else if rm == 5 && md == 0 {
            mr.disp = self.u32()?;
        } else {
            mr.base = Some(rm);
        }
        Ok(mr)
    }

    /// The SIB byte: scaled index, and base (or a disp32 instead).
    fn sib(&mut self, md: u8, mr: &mut MemRef) -> Result<(), Fault> {
        let sib = self.u8()?;
        let (ss, idx, base) = (sib >> 6, (sib >> 3) & 7, sib & 7);
        if idx != 4 {
            mr.index = Some(idx);
            mr.scale = 1 << ss;
        }
        if base == 5 && md == 0 {
            mr.disp = self.u32()?;
        } else {
            mr.base = Some(base);
        }
        Ok(())
    }
}
