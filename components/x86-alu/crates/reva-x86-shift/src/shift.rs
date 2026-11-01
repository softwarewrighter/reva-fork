//! `shl`, `shr`, `sar`, and the dispatch of the shift group.

use crate::rotate::Rotates;
use reva_x86_ops::operands::Operands;
use reva_x86_types::cpu::Cpu;
use reva_x86_types::op::Shift;
use reva_x86_types::operand::Size;

reva_macros::ext_trait! {
    impl Cpu;
    /// The shift group; `cnt` is 1-31 (the caller skips a zero count).
    pub trait Shifts: Operands, Rotates {
        /// Shift or rotate `v` by `cnt`, setting the flags.
        fn shift(&mut self, op: Shift, size: Size, v: u32, cnt: u32) -> u32 {
            let v = v & size.mask();
            match op {
                Shift::Shl => self.shl(size, v, cnt),
                Shift::Shr => self.shr(size, v, cnt),
                Shift::Sar => self.sar(size, v, cnt),
                Shift::Rol | Shift::Ror => self.rotate(op, size, v, cnt),
                Shift::Rcl | Shift::Rcr => self.rotate_carry(op, size, v, cnt),
            }
        }

        /// `shl`.
        fn shl(&mut self, size: Size, v: u32, cnt: u32) -> u32 {
            let full = (v as u64) << cnt;
            let r = full as u32 & size.mask();
            self.cf = (full >> (size.bytes() * 8)) & 1 != 0;
            self.of = (r & size.sign() != 0) != self.cf;
            self.set_szp(r, size);
            r
        }

        /// `shr`.
        fn shr(&mut self, size: Size, v: u32, cnt: u32) -> u32 {
            let r = ((v as u64) >> cnt) as u32;
            self.cf = ((v as u64) >> (cnt - 1)) & 1 != 0;
            self.of = v & size.sign() != 0;
            self.set_szp(r, size);
            r
        }

        /// `sar`.
        fn sar(&mut self, size: Size, v: u32, cnt: u32) -> u32 {
            let (m, s) = (size.mask(), size.sign());
            let sv = if v & s != 0 {
                (v as i64) - ((m as i64) + 1)
            } else {
                v as i64
            };
            let r = (sv >> cnt.min(63)) as u32 & m;
            self.cf = (sv >> (cnt - 1).min(63)) & 1 != 0;
            self.of = false;
            self.set_szp(r, size);
            r
        }
    }
}
