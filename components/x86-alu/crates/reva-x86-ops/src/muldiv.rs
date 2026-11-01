//! `mul`, `imul`, `div`, `idiv` on the accumulator pair.

use crate::regs::Regs;
use reva_x86_base::fault::Fault;
use reva_x86_types::cpu::Cpu;
use reva_x86_types::op::MulDiv;
use reva_x86_types::operand::Size;

/// `v`'s low `b` bits, sign-extended.
pub fn sext(v: u64, b: u32) -> i64 {
    ((v << (64 - b)) as i64) >> (64 - b)
}

/// `div`/`idiv` of the pair `hi:lo` by `src`: (quotient, remainder), or
/// `None` for a divide error (zero, or a quotient that does not fit).
pub fn divide(op: MulDiv, size: Size, lo: u32, hi: u32, src: u32) -> Option<(u64, u64)> {
    let bits = size.bytes() * 8;
    let m = size.mask() as u64;
    if op == MulDiv::Div {
        let n = ((hi as u64) << bits) | lo as u64;
        if src == 0 {
            return None;
        }
        let q = n / src as u64;
        return (q <= m).then_some((q, n % src as u64));
    }
    let n = sext(((hi as u64) << bits) | lo as u64, bits * 2);
    let d = sext(src as u64, bits);
    let q = n.checked_div(d)?;
    (sext(q as u64 & m, bits) == q).then_some((q as u64 & m, (n % d) as u64 & m))
}

reva_macros::ext_trait! {
    impl Cpu;
    /// Multiply and divide.
    pub trait MulDivs: Regs {
        /// `mul`/`imul`/`div`/`idiv` of the accumulator by `src`; `at` is the
        /// instruction's address, for the divide error.
        fn muldiv(&mut self, op: MulDiv, size: Size, src: u32, at: u32) -> Result<(), Fault> {
            let (lo, hi) = self.accumulators(size);
            let (q, r) = match op {
                MulDiv::Mul | MulDiv::Imul => self.multiply(op, size, lo, src),
                _ => divide(op, size, lo, hi, src).ok_or(Fault::DivideError { eip: at })?,
            };
            self.store_pair(size, q, r);
            Ok(())
        }

        /// `mul`/`imul`: (low, high) of the product; CF = OF = it overflowed.
        fn multiply(&mut self, op: MulDiv, size: Size, lo: u32, src: u32) -> (u64, u64) {
            let bits = size.bytes() * 8;
            let m = size.mask() as u64;
            let (q, r, overflow) = if op == MulDiv::Mul {
                let p = lo as u64 * src as u64;
                let (q, r) = (p & m, (p >> bits) & m);
                (q, r, r != 0)
            } else {
                let p = sext(lo as u64, bits) * sext(src as u64, bits);
                let fits = sext(p as u64 & m, bits) == p;
                (p as u64 & m, (p as u64 >> bits) & m, !fits)
            };
            (self.cf, self.of) = (overflow, overflow);
            (q, r)
        }
    }
}
