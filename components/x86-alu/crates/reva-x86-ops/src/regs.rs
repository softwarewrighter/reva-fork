//! Registers: effective addresses, 8/16/32-bit views, accumulator pairs.

use reva_x86_types::cpu::{Cpu, EAX, EDX};
use reva_x86_types::operand::{MemRef, Size};

reva_macros::ext_trait! {
    impl Cpu;
    /// Register access.
    pub trait Regs {
        /// `[base + index*scale + disp]`.
        #[inline]
        fn ea(&self, m: &MemRef) -> u32 {
            let mut a = m.disp;
            if let Some(b) = m.base {
                a = a.wrapping_add(self.r[b as usize]);
            }
            if let Some(i) = m.index {
                a = a.wrapping_add(self.r[i as usize].wrapping_mul(m.scale as u32));
            }
            a
        }

        /// Register `n` at `size`; for bytes 4-7 are AH CH DH BH.
        #[inline]
        fn get_reg(&self, n: u8, size: Size) -> u32 {
            let n = n as usize;
            match size {
                Size::D => self.r[n],
                Size::W => self.r[n] & 0xffff,
                Size::B if n < 4 => self.r[n] & 0xff,
                Size::B => (self.r[n - 4] >> 8) & 0xff,
            }
        }

        /// Set register `n` at `size`, keeping the rest of the register.
        #[inline]
        fn set_reg(&mut self, n: u8, size: Size, v: u32) {
            let n = n as usize;
            match size {
                Size::D => self.r[n] = v,
                Size::W => self.r[n] = (self.r[n] & 0xffff_0000) | (v & 0xffff),
                Size::B if n < 4 => self.r[n] = (self.r[n] & !0xff) | (v & 0xff),
                Size::B => self.r[n - 4] = (self.r[n - 4] & !0xff00) | ((v & 0xff) << 8),
            }
        }

        /// The accumulator pair of `mul`/`div`: AL/AH, DX:AX, EDX:EAX (low, high).
        fn accumulators(&self, size: Size) -> (u32, u32) {
            match size {
                Size::B => (self.r[0] & 0xff, (self.r[0] >> 8) & 0xff),
                Size::W => (self.r[0] & 0xffff, self.r[2] & 0xffff),
                Size::D => (self.r[0], self.r[2]),
            }
        }

        /// Store a `mul`/`div` result into the accumulator pair.
        fn store_pair(&mut self, size: Size, q: u64, r: u64) {
            match size {
                Size::B => self.set_reg(EAX, Size::W, (q as u32) | ((r as u32) << 8)),
                Size::W => {
                    self.set_reg(EAX, Size::W, q as u32);
                    self.set_reg(EDX, Size::W, r as u32);
                }
                Size::D => {
                    self.r[0] = q as u32;
                    self.r[2] = r as u32;
                }
            }
        }
    }
}
