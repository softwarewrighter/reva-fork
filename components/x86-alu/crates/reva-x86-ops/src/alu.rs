//! The eight ALU operations, `push`, `pop`.

use crate::operands::Operands;
use reva_x86_base::fault::Fault;
use reva_x86_mem::memory::Memory;
use reva_x86_types::cpu::{Cpu, ESP};
use reva_x86_types::op::Alu;
use reva_x86_types::operand::Size;

/// `a + b + carry` within `mask`: (result, CF, OF).
pub fn add_flags(a: u32, b: u32, carry: bool, mask: u32, sign: u32) -> (u32, bool, bool) {
    let full = a as u64 + b as u64 + carry as u64;
    let r = full as u32 & mask;
    (r, full > mask as u64, (a ^ r) & (b ^ r) & sign != 0)
}

/// `a - b - borrow` within `mask`: (result, CF, OF).
pub fn sub_flags(a: u32, b: u32, borrow: bool, mask: u32, sign: u32) -> (u32, bool, bool) {
    let c = borrow as u32;
    let r = a.wrapping_sub(b).wrapping_sub(c) & mask;
    (
        r,
        (a as u64) < b as u64 + c as u64,
        (a ^ b) & (a ^ r) & sign != 0,
    )
}

reva_macros::ext_trait! {
    impl Cpu;
    /// Arithmetic and the return stack.
    pub trait Arith: Operands {
        /// The eight ALU operations with full flag semantics.
        fn alu(&mut self, op: Alu, size: Size, a: u32, b: u32) -> u32 {
            let (m, s) = (size.mask(), size.sign());
            let (a, b) = (a & m, b & m);
            let (r, cf, of) = match op {
                Alu::Add | Alu::Adc => add_flags(a, b, op == Alu::Adc && self.cf, m, s),
                Alu::Sub | Alu::Sbb | Alu::Cmp => sub_flags(a, b, op == Alu::Sbb && self.cf, m, s),
                Alu::And => (a & b, false, false),
                Alu::Or => (a | b, false, false),
                Alu::Xor => (a ^ b, false, false),
            };
            (self.cf, self.of) = (cf, of);
            self.set_szp(r, size);
            r
        }

        /// Push a cell onto the return stack.
        #[inline]
        fn push32(&mut self, mem: &mut Memory, v: u32) -> Result<(), Fault> {
            let sp = self.r[ESP as usize].wrapping_sub(4);
            if sp < self.esp_floor {
                return Err(Fault::ReturnStackOverflow);
            }
            mem.write_u32(sp, v)?;
            self.r[ESP as usize] = sp;
            Ok(())
        }

        /// Pop a cell from the return stack.
        #[inline]
        fn pop32(&mut self, mem: &Memory) -> Result<u32, Fault> {
            let sp = self.r[ESP as usize];
            let v = mem.read_u32(sp)?;
            self.r[ESP as usize] = sp.wrapping_add(4);
            Ok(v)
        }
    }
}
