//! Functions of st(0) (and st(1)): unary operations, logarithms,
//! trigonometry, `fxtract`, `fprem`.

use reva_x86_float::math::split;
use reva_x86_float::values::TRIG_LIMIT;
use reva_x86_fpustate::fpu::{C2, IE, INDEFINITE, ZE};
use reva_x86_types::cpu::Cpu;
use reva_x86_types::fpu_op::{FUnary, FpuOp};

reva_macros::ext_trait! {
    impl Cpu;
    /// The x87's mathematical functions.
    pub trait FpuMath {
        /// An operation on st(0) in place.
        fn unary(&mut self, u: FUnary) {
            let f = &mut self.fpu;
            let v = f.st(0);
            let trig = matches!(u, FUnary::Sin | FUnary::Cos);
            if trig && v.abs() >= TRIG_LIMIT {
                f.sw |= C2;
                return;
            }
            let r = match u {
                FUnary::Chs => -v,
                FUnary::Abs => v.abs(),
                FUnary::Sqrt => v.sqrt(),
                FUnary::Rndint => f.round(v),
                FUnary::F2xm1 => (v * std::f64::consts::LN_2).exp_m1(),
                FUnary::Sin => v.sin(),
                FUnary::Cos => v.cos(),
            };
            if r.is_nan() && !v.is_nan() {
                f.sw |= IE;
            }
            if trig {
                f.sw &= !C2;
            }
            f.set(0, r);
        }

        /// `fyl2x` (`y * log2 x`) and `fyl2xp1` (`y * log2(x + 1)`), popping.
        fn log2(&mut self, op: FpuOp) {
            let f = &mut self.fpu;
            let (x, y) = (f.st(0), f.st(1));
            let l = if op == FpuOp::Yl2x {
                x.log2()
            } else {
                x.ln_1p() / std::f64::consts::LN_2
            };
            if x == 0.0 && op == FpuOp::Yl2x {
                f.sw |= ZE;
            }
            f.set(1, y * l);
            f.pop();
        }

        /// `fptan` (tan, then 1.0 pushed) and `fsincos` (sin, cos pushed).
        fn trig(&mut self, op: FpuOp) {
            let f = &mut self.fpu;
            let v = f.st(0);
            if v.abs() >= TRIG_LIMIT {
                f.sw |= C2;
                return;
            }
            f.sw &= !C2;
            if op == FpuOp::Ptan {
                f.set(0, v.tan());
                f.push(1.0);
            } else {
                f.set(0, v.sin());
                f.push(v.cos());
            }
        }

        /// `fxtract`: exponent into st(0), then the significand pushed.
        fn xtract(&mut self) {
            let f = &mut self.fpu;
            let v = f.st(0);
            let (sig, exp) = if v == 0.0 {
                f.sw |= ZE;
                (v, f64::NEG_INFINITY)
            } else if !v.is_finite() {
                (v, v.abs())
            } else {
                let (s, e) = split(v.abs());
                (s.copysign(v), e as f64)
            };
            f.set(0, exp);
            f.push(sig);
        }

        /// `fprem`/`fprem1`: the partial remainder, with the quotient's
        /// low bits in C0 C3 C1.
        fn prem(&mut self, op: FpuOp) {
            let f = &mut self.fpu;
            let (a, b) = (f.st(0), f.st(1));
            if b == 0.0 || !a.is_finite() || b.is_nan() {
                f.sw |= IE;
                f.set(0, INDEFINITE);
                f.set_cc(false, false, false, false);
                return;
            }
            let mut r = a % b; // exact, the sign of `a`, as fprem
            let mut q = ((a - r) / b).abs();
            if op == FpuOp::Prem1 {
                let (ar, ab) = (r.abs(), b.abs());
                let odd = (q % 2.0) == 1.0;
                if 2.0 * ar > ab || (2.0 * ar == ab && odd) {
                    r -= ab.copysign(r);
                    q += 1.0;
                }
            }
            let q = if q < TRIG_LIMIT { q as u64 } else { 0 };
            f.set(0, r);
            f.set_cc(q & 2 != 0, false, q & 1 != 0, q & 4 != 0);
        }
    }
}
