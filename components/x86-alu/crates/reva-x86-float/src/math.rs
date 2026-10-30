//! Scaling, splitting, comparing and classifying doubles.

/// `v * 2^n` without overflowing the intermediate power.
pub fn ldexp(mut v: f64, mut n: i64) -> f64 {
    let step = |k: i64| f64::from_bits(((k + 1023) as u64) << 52);
    while n > 1000 {
        v *= step(1000);
        n -= 1000;
        if v.is_infinite() {
            return v;
        }
    }
    while n < -1000 {
        v *= step(-1000);
        n += 1000;
        if v == 0.0 {
            return v;
        }
    }
    v * step(n)
}

/// `(significand in [1, 2), unbiased exponent)` of a finite non-zero value.
pub fn split(v: f64) -> (f64, i64) {
    let bits = v.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i64;
    if exp == 0 {
        let (s, e) = split(v * f64::from_bits((1023 + 64) << 52));
        return (s, e - 64);
    }
    let sig = f64::from_bits((bits & !(0x7ff << 52)) | (1023 << 52));
    (sig, exp - 1023)
}

/// Condition codes of a comparison: (C3, C2, C0).
pub fn compare(a: f64, b: f64) -> (bool, bool, bool) {
    if a.is_nan() || b.is_nan() {
        (true, true, true)
    } else if a > b {
        (false, false, false)
    } else if a < b {
        (false, false, true)
    } else {
        (true, false, false)
    }
}

/// `fxam`'s class of a register: (C3, C2, C0).
pub fn classify(v: f64, empty: bool) -> (bool, bool, bool) {
    if empty {
        (true, false, true)
    } else if v.is_nan() {
        (false, false, true)
    } else if v.is_infinite() {
        (false, true, true)
    } else if v == 0.0 {
        (true, false, false)
    } else if v.is_subnormal() {
        (true, true, false)
    } else {
        (false, true, false)
    }
}

/// `fscale`: `v * 2^trunc(s)` (a NaN scale is the result).
pub fn scale_value(v: f64, s: f64) -> f64 {
    if s.is_nan() {
        s
    } else {
        ldexp(v, s.trunc().clamp(-1e5, 1e5) as i64)
    }
}
