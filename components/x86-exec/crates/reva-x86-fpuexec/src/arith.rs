//! Arithmetic, comparisons, `fxam`.

use crate::memory::FpuMemory;
use reva_x86_base::fault::Fault;
use reva_x86_float::math::{classify, compare};
use reva_x86_float::values::arith_value;
use reva_x86_fpustate::fpu::{C1, IE, ZE};
use reva_x86_mem::memory::Memory;
use reva_x86_types::cpu::Cpu;
use reva_x86_types::fpu_op::{FArith, FSrc};

type R = Result<(), Fault>;

reva_macros::ext_trait! {
    impl Cpu;
    /// The x87's arithmetic and comparisons.
    pub trait FpuArith: FpuMemory {
        /// `st(dst) = st(dst) op src`, then pop if asked.
        fn arith(&mut self, mem: &Memory, op: FArith, dst: u8, src: FSrc, pop: bool) -> R {
            let s = self.fload(mem, src)?;
            let f = &mut self.fpu;
            let d = f.st(dst);
            let (r, zero_divide) = arith_value(op, d, s);
            if zero_divide {
                f.sw |= ZE;
            }
            if r.is_nan() && !d.is_nan() && !s.is_nan() {
                f.sw |= IE;
            }
            f.set(dst, r);
            if pop {
                f.pop();
            }
            Ok(())
        }

        /// `fcom`/`fucom` (and `p`, `pp` forms): st(0) against `src`.
        fn compare_op(&mut self, mem: &Memory, src: FSrc, pops: u8, unordered: bool) -> R {
            let b = self.fload(mem, src)?;
            let f = &mut self.fpu;
            let a = f.st(0);
            let (c3, c2, c0) = compare(a, b);
            if c2 && !unordered {
                f.sw |= IE;
            }
            f.set_cc(c3, c2, false, c0);
            for _ in 0..pops {
                f.pop();
            }
            Ok(())
        }

        /// `fcomi`/`fucomi` (and `p`): the result goes to EFLAGS.
        fn comi(&mut self, i: u8, pop: bool, unordered: bool) {
            let f = &mut self.fpu;
            let (a, b) = (f.st(0), f.st(i));
            let (z, p, c) = compare(a, b);
            if p && !unordered {
                f.sw |= IE;
            }
            f.sw &= !C1;
            if pop {
                f.pop();
            }
            (self.zf, self.pf, self.cf, self.of, self.sf) = (z, p, c, false, false);
        }

        /// `fxam`: the class of st(0) in C3 C2 C0, its sign in C1.
        fn xam(&mut self) {
            let f = &mut self.fpu;
            let v = f.regs[f.phys(0)];
            let empty = f.is_empty(0);
            let (c3, c2, c0) = classify(v, empty);
            f.set_cc(c3, c2, v.is_sign_negative() && !empty, c0);
        }
    }
}
