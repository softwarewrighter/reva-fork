//! The x87 80-bit format.

/// `f64` → the x87 80-bit format (exact).
pub fn f64_to_f80(v: f64) -> [u8; 10] {
    let bits = v.to_bits();
    let sign = ((bits >> 63) as u16) << 15;
    let exp = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1 << 52) - 1);
    let (e, m) = match exp {
        0x7ff => (0x7fff, (1 << 63) | (frac << 11)),
        0 if frac == 0 => (0, 0),
        0 => {
            // f64 subnormal: normal in the wider exponent range
            let shift = frac.leading_zeros();
            ((16383 - 1074 + 63 - shift as i32) as u16, frac << shift)
        }
        _ => ((exp - 1023 + 16383) as u16, (1 << 63) | (frac << 11)),
    };
    let mut out = [0; 10];
    out[..8].copy_from_slice(&m.to_le_bytes());
    out[8..].copy_from_slice(&(sign | e).to_le_bytes());
    out
}

/// `m >> sh`, rounded to nearest, ties to even.
pub fn shr_round(m: u64, sh: u32) -> u64 {
    match sh {
        0 => m,
        1..=63 => {
            let (q, rem, half) = (m >> sh, m & ((1 << sh) - 1), 1u64 << (sh - 1));
            q + u64::from(rem > half || (rem == half && q & 1 == 1))
        }
        64 => u64::from(m > 1 << 63),
        _ => 0,
    }
}

/// The x87 80-bit format → `f64`, rounded to nearest-even.
pub fn f80_to_f64(b: &[u8; 10]) -> f64 {
    let m = u64::from_le_bytes(b[..8].try_into().unwrap());
    let se = u16::from_le_bytes([b[8], b[9]]);
    let sign = u64::from(se >> 15) << 63;
    let e = i32::from(se & 0x7fff);
    if e == 0x7fff {
        if m << 1 == 0 {
            return f64::from_bits(sign | (0x7ff << 52));
        }
        let frac = ((m >> 11) & ((1 << 52) - 1)).max(1);
        return f64::from_bits(sign | (0x7ff << 52) | frac | (1 << 51));
    }
    if m == 0 {
        return f64::from_bits(sign);
    }
    f80_finite(sign, e, m)
}

/// A finite non-zero 80-bit value (`m` != 0) as `f64`: normal, subnormal
/// or overflowing to infinity.
fn f80_finite(sign: u64, e: i32, m: u64) -> f64 {
    // value = m * 2^(e' - 16383 - 63), e' = 1 for denormals
    let lz = m.leading_zeros();
    let m = m << lz;
    let mut exp2 = e.max(1) - 16383 - lz as i32; // m now has its top bit set
    if exp2 < -1022 {
        // subnormal in f64 (a carry into bit 52 is the smallest normal)
        let sh = (-1011 - exp2) as u32;
        return f64::from_bits(sign | shr_round(m, sh));
    }
    let mut q = shr_round(m, 11);
    if q == 1 << 53 {
        q >>= 1;
        exp2 += 1;
    }
    if exp2 > 1023 {
        return f64::from_bits(sign | (0x7ff << 52));
    }
    f64::from_bits(sign | (((exp2 + 1023) as u64) << 52) | (q & ((1 << 52) - 1)))
}
