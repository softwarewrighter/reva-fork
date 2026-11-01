//! Pushing and popping cells: the data stack's `upsh`/`upop`/`drop`
//! macros of newcore.asm, and the return stack.

use crate::fault::Fault;
use crate::machine::Machine;
use crate::x86::{Arith, ESI};

const SI: usize = ESI as usize;

type R<T = ()> = Result<T, Fault>;

impl Machine {
    #[inline]
    pub fn upsh(&mut self, v: u32) -> R {
        self.dup()?;
        self.set_eax(v);
        Ok(())
    }

    #[inline]
    pub fn drop1(&mut self) -> R {
        let sp = self.cpu.r[SI];
        let v = self.mem.read_u32(sp)?;
        self.set_eax(v);
        self.cpu.r[SI] = sp.wrapping_add(4);
        Ok(())
    }

    #[inline]
    pub fn upop(&mut self) -> R<u32> {
        let v = self.eax();
        self.drop1()?;
        Ok(v)
    }

    #[inline]
    pub fn rpush(&mut self, v: u32) -> R {
        self.cpu.push32(&mut self.mem, v)
    }

    #[inline]
    pub fn rpop(&mut self) -> R<u32> {
        self.cpu.pop32(&self.mem)
    }
}
