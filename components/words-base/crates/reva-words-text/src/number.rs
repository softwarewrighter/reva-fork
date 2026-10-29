//! `single`, `double` and `(.r)`: numbers to and from text.

use crate::digits::{Prefix, digit_value, number_prefix, sign_split};

/// `PROC single`: `[-][#$%&]digits` or `'c`, in `base`.  Overflow is not
/// a number, as in the original (`jo .err`).
pub fn parse_single(s: &[u8], base: u32) -> Option<u32> {
    let (neg, rest) = sign_split(s)?;
    let sign = |v: u32| if neg { v.wrapping_neg() } else { v };
    match number_prefix(rest, base, true)? {
        Prefix::Char(c) => Some(sign(c)),
        Prefix::Digits(base, digits) => single_digits(digits, base).map(sign),
    }
}

/// The digits of a single: 32-bit, a signed overflow fails.
pub fn single_digits(digits: &[u8], base: u32) -> Option<u32> {
    digits.iter().try_fold(0u32, |acc, &c| {
        let d = digit_value(c as u32, base)?;
        let full = acc as u64 * base as u64;
        if full > u32::MAX as u64 {
            return None;
        }
        let (sum, overflow) = (full as u32 as i32).overflowing_add(d as i32);
        (!overflow).then_some(sum as u32)
    })
}

/// `PROC double`: like `single` with a trailing `L`; `.` and `,` are
/// ignored as separators; 64-bit wrap-around.  Low and high cells.
pub fn parse_double(s: &[u8], base: u32) -> Option<(u32, u32)> {
    let s = s.strip_suffix(b"L").filter(|s| !s.is_empty())?;
    let (neg, rest) = sign_split(s)?;
    let Prefix::Digits(base, digits) = number_prefix(rest, base, false)? else {
        return None;
    };
    let v = double_digits(digits, base)?;
    let v = if neg { v.wrapping_neg() } else { v };
    Some((v as u32, (v >> 32) as u32))
}

/// The digits of a double, skipping `.` and `,`.
pub fn double_digits(digits: &[u8], base: u32) -> Option<u64> {
    digits
        .iter()
        .filter(|&&c| c != b'.' && c != b',')
        .try_fold(0u64, |acc, &c| {
            let d = digit_value(c as u32, base)?;
            Some(acc.wrapping_mul(base as u64).wrapping_add(d as u64))
        })
}

/// `PROC _printr` (`(.r)`): `n` in `base` (2 or more; signed only in base
/// 10), right-aligned to `width` with `pad`.
pub fn format_r(n: u32, base: u32, width: u32, pad: u8) -> Vec<u8> {
    let neg = base == 10 && (n as i32) < 0;
    let mut v = if neg { n.wrapping_neg() } else { n };
    let mut out = Vec::new();
    loop {
        let mut c = (v % base) as u8 + b'0';
        if c > b'9' {
            c += 7;
        }
        out.push(c);
        v /= base;
        if v == 0 {
            break;
        }
    }
    if neg {
        out.push(b'-');
    }
    out.resize(out.len().max(width as usize), pad);
    out.reverse();
    out
}
