//! Registers by number, and result buffers in the heap.

use reva_core::fault::Fault;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Registers, and heap buffers whose address a statics cell keeps.
    pub trait MachineRegs {
        /// Register `r` (`reva_core::x86::EAX` ...).
        fn reg(&self, r: u8) -> u32 {
            self.cpu.r[r as usize]
        }

        fn set_reg(&mut self, r: u8, v: u32) {
            self.cpu.r[r as usize] = v;
        }

        /// A heap buffer of at least `len` bytes, kept in statics cells
        /// `buf`/`cap` (so it survives in saved images).
        fn ffi_buffer(&mut self, buf: u32, cap: Option<u32>, len: u32) -> R<u32> {
            let have = self.mem.read_u32(buf)?;
            let have_len = match cap {
                Some(c) => self.mem.read_u32(c)?,
                None => len,
            };
            if have != 0 && have_len >= len {
                return Ok(have);
            }
            if have != 0 {
                self.heap.free(have);
            }
            let size = len.max(64);
            let p = self.heap.alloc(size).ok_or(Fault::OutOfSpace("heap"))?;
            self.mem.write_u32(buf, p)?;
            if let Some(c) = cap {
                self.mem.write_u32(c, size)?;
            }
            Ok(p)
        }
    }
}
