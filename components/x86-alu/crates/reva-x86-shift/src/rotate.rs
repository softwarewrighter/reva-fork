//! `rol`, `ror`, `rcl`, `rcr`.

use reva_x86_types::cpu::Cpu;
use reva_x86_types::op::Shift;
use reva_x86_types::operand::Size;

reva_macros::ext_trait! {
    impl Cpu;
    /// Rotates (they leave ZF, SF and PF alone).
    pub trait Rotates {
        /// `rol`/`ror`.
        fn rotate(&mut self, op: Shift, size: Size, v: u32, cnt: u32) -> u32 {
            let bits = size.bytes() * 8;
            let (m, s) = (size.mask(), size.sign());
            let c = cnt % bits;
            let r = if c == 0 {
                v
            } else if op == Shift::Rol {
                ((v << c) | (v >> (bits - c))) & m
            } else {
                ((v >> c) | (v << (bits - c))) & m
            };
            if op == Shift::Rol {
                self.cf = r & 1 != 0;
                self.of = (r & s != 0) != self.cf;
            } else {
                self.cf = r & s != 0;
                self.of = (r & s != 0) != (r & (s >> 1) != 0);
            }
            r
        }

        /// `rcl`/`rcr`: through the carry, one bit at a time.
        fn rotate_carry(&mut self, op: Shift, size: Size, v: u32, cnt: u32) -> u32 {
            let bits = size.bytes() * 8;
            let (m, s) = (size.mask(), size.sign());
            let mut r = v;
            for _ in 0..cnt % (bits + 1) {
                let out = if op == Shift::Rcl { r & s != 0 } else { r & 1 != 0 };
                r = if op == Shift::Rcl {
                    ((r << 1) | self.cf as u32) & m
                } else {
                    (r >> 1) | if self.cf { s } else { 0 }
                };
                self.cf = out;
            }
            self.of = if op == Shift::Rcl {
                (r & s != 0) != self.cf
            } else {
                (r & s != 0) != (r & (s >> 1) != 0)
            };
            r
        }
    }
}
