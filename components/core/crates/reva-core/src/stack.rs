//! The top of the data stack: EAX is TOS, `[ESI]` is NOS (newcore.asm's
//! register conventions).

use crate::fault::Fault;
use crate::machine::Machine;
use crate::x86::{EAX, ESI};

const SI: usize = ESI as usize;

type R<T = ()> = Result<T, Fault>;

/// TOS and NOS, shared by the gateway routines (reva-words) and the
/// virtual libraries (reva-vlib).
impl Machine {
    #[inline]
    pub fn eax(&self) -> u32 {
        self.cpu.r[EAX as usize]
    }

    #[inline]
    pub fn set_eax(&mut self, v: u32) {
        self.cpu.r[EAX as usize] = v;
    }

    #[inline]
    pub fn nos(&self) -> R<u32> {
        self.mem.read_u32(self.cpu.r[SI])
    }

    #[inline]
    pub fn set_nos(&mut self, v: u32) -> R {
        self.mem.write_u32(self.cpu.r[SI], v)
    }

    #[inline]
    pub fn dup(&mut self) -> R {
        let sp = self.cpu.r[SI].wrapping_sub(4);
        self.mem.write_u32(sp, self.eax())?;
        self.cpu.r[SI] = sp;
        Ok(())
    }
}
