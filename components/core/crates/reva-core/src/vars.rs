//! Static variables: reading, writing, and the two read most often.

use crate::fault::Fault;
use crate::machine::Machine;
use crate::machine::st;

type R<T = ()> = Result<T, Fault>;

impl Machine {
    #[inline]
    pub fn setv(&mut self, a: u32, v: u32) -> R {
        self.mem.write_u32(a, v)
    }

    /// Read a static variable (see [`st`]).
    pub fn var(&self, addr: u32) -> u32 {
        self.mem.read_u32(addr).unwrap_or(0)
    }

    #[inline]
    pub fn compiling(&self) -> bool {
        self.var(st::IS_COMPILING) != 0
    }

    #[inline]
    pub fn here(&self) -> u32 {
        self.var(st::H)
    }
}
