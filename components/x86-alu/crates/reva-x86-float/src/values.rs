//! The constants and the arithmetic forms.

use reva_x86_types::fpu_op::{FArith, FConst};

/// Inputs at or beyond 2^63 are out of range for `fsin` and friends: C2
/// is set and the operand is left alone.  Also the 64-bit integer range.
pub const TRIG_LIMIT: f64 = (1u64 << 63) as f64;

/// The value of an `fld` constant.
pub fn constant(c: FConst) -> f64 {
    match c {
        FConst::One => 1.0,
        FConst::L2t => std::f64::consts::LOG2_10,
        FConst::L2e => std::f64::consts::LOG2_E,
        FConst::Pi => std::f64::consts::PI,
        FConst::Lg2 => std::f64::consts::LOG10_2,
        FConst::Ln2 => std::f64::consts::LN_2,
        FConst::Zero => 0.0,
    }
}

/// `d op s` (the `r` forms reversed): the result, and whether it divided
/// a finite non-zero value by zero (ZE).
pub fn arith_value(op: FArith, d: f64, s: f64) -> (f64, bool) {
    let (num, den) = match op {
        FArith::Div => (d, s),
        FArith::Divr => (s, d),
        _ => (1.0, 1.0),
    };
    let zero_divide = den == 0.0 && num.is_finite() && num != 0.0;
    let r = match op {
        FArith::Add => d + s,
        FArith::Mul => d * s,
        FArith::Sub => d - s,
        FArith::Subr => s - d,
        FArith::Div | FArith::Divr => num / den,
    };
    (r, zero_divide)
}
