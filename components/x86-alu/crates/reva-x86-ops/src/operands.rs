//! Reading and writing operands; the result flags.

use crate::regs::Regs;
use reva_x86_base::fault::Fault;
use reva_x86_mem::memory::Memory;
use reva_x86_types::cpu::Cpu;
use reva_x86_types::operand::{Opd, Size};

reva_macros::ext_trait! {
    impl Cpu;
    /// Operands.
    pub trait Operands: Regs {
        /// The value of operand `o` at `size`.
        #[inline]
        fn read(&self, mem: &Memory, o: Opd, size: Size) -> Result<u32, Fault> {
            match o {
                Opd::Reg(n) => Ok(self.get_reg(n, size)),
                Opd::Imm(v) => Ok(v & size.mask()),
                Opd::Mem(m) => {
                    let a = self.ea(&m);
                    match size {
                        Size::B => mem.read_u8(a).map(u32::from),
                        Size::W => mem.read_u16(a).map(u32::from),
                        Size::D => mem.read_u32(a),
                    }
                }
            }
        }

        /// Store `v` into operand `o` at `size`.
        #[inline]
        fn write(&mut self, mem: &mut Memory, o: Opd, size: Size, v: u32) -> Result<(), Fault> {
            match o {
                Opd::Reg(n) => {
                    self.set_reg(n, size, v);
                    Ok(())
                }
                Opd::Mem(m) => {
                    let a = self.ea(&m);
                    match size {
                        Size::B => mem.write_u8(a, v as u8),
                        Size::W => mem.write_u16(a, v as u16),
                        Size::D => mem.write_u32(a, v),
                    }
                }
                Opd::Imm(_) => Err(Fault::Trap {
                    eip: self.eip,
                    what: "write to immediate",
                }),
            }
        }

        /// ZF, SF and PF of a result.
        #[inline]
        fn set_szp(&mut self, r: u32, size: Size) {
            self.zf = r & size.mask() == 0;
            self.sf = r & size.sign() != 0;
            self.pf = (r as u8).count_ones().is_multiple_of(2);
        }
    }
}
