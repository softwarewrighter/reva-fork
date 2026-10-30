//! Moves, the ALU forms, `xchg`, `inc`/`dec`, `imul`.

use reva_x86_base::fault::Fault;
use reva_x86_mem::memory::Memory;
use reva_x86_ops::alu::Arith;
use reva_x86_ops::operands::Operands;
use reva_x86_types::cpu::Cpu;
use reva_x86_types::op::Alu;
use reva_x86_types::operand::{Opd, Size};

type R = Result<(), Fault>;

reva_macros::ext_trait! {
    impl Cpu;
    /// Instructions on operands.
    pub trait DataOps: Operands, Arith {
        /// `op dst, src` (`cmp` writes nothing).
        #[inline(always)]
        fn alu_insn(&mut self, mem: &mut Memory, op: Alu, size: Size, dst: Opd, src: Opd) -> R {
            let a = self.read(mem, dst, size)?;
            let b = self.read(mem, src, size)?;
            let r = self.alu(op, size, a, b);
            if op != Alu::Cmp {
                self.write(mem, dst, size, r)?;
            }
            Ok(())
        }

        /// `movzx`/`movsx`.
        fn movx(&mut self, mem: &Memory, signed: bool, from: Size, dst: u8, src: Opd) -> R {
            let v = self.read(mem, src, from)?;
            self.r[dst as usize] = if signed && v & from.sign() != 0 {
                v | !from.mask()
            } else {
                v
            };
            Ok(())
        }

        /// `xchg a, b`.
        fn xchg(&mut self, mem: &mut Memory, size: Size, a: Opd, b: Opd) -> R {
            let x = self.read(mem, a, size)?;
            let y = self.read(mem, b, size)?;
            self.write(mem, a, size, y)?;
            self.write(mem, b, size, x)
        }

        /// `inc` (`Alu::Add`) / `dec` (`Alu::Sub`): CF is kept.
        fn inc_dec(&mut self, mem: &mut Memory, op: Alu, size: Size, dst: Opd) -> R {
            let v = self.read(mem, dst, size)?;
            let c = self.cf;
            let r = self.alu(op, size, v, 1);
            self.cf = c;
            self.write(mem, dst, size, r)
        }

        /// `imul dst, src[, imm]`: the low 32 bits; CF = OF = they lost bits.
        fn imul(&mut self, mem: &Memory, dst: u8, src: Opd, imm: Option<u32>) -> R {
            let s = self.read(mem, src, Size::D)? as i32 as i64;
            let o = match imm {
                Some(i) => i as i32 as i64,
                None => self.r[dst as usize] as i32 as i64,
            };
            let p = s * o;
            let fits = p == p as i32 as i64;
            (self.cf, self.of) = (!fits, !fits);
            self.r[dst as usize] = p as u32;
            Ok(())
        }
    }
}
