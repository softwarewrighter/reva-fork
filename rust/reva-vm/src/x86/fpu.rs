//! # x87: the floating-point unit Reva's float library drives
//!
//! `lib/math/floats` (and CoSy, which keeps its floats as 80-bit values)
//! writes x87 instructions directly as bytes: `f+` is `inline{ de c1 }`,
//! `f@` is `fld tbyte [eax]`.  This module gives the vx86 VM that unit:
//! an eight-register stack with tags, `TOP`, the control word (rounding
//! mode) and the status word (condition codes `C0`–`C3`, sticky exception
//! flags), with every exception masked, as Reva left them.
//!
//! **Precision.** Registers hold `f64`, not 80-bit extended values.  Loads
//! and stores of the 80-bit memory format (`fld`/`fstp tbyte`) are exact
//! in the direction f64 → f80 and round to nearest-even in the direction
//! f80 → f64; 64-bit integers convert with f64 rounding; arithmetic rounds
//! to f64 whatever the precision-control field says, and the rounding
//! control field applies to integer conversion (`fist`, `frndint`) only.
//! Results can therefore differ from a real x87 in the last bits of the
//! extended format; for values that fit a double they are the IEEE double
//! results.  `fsin`, `fyl2x` and friends use Rust's `f64` functions.
//!
//! Not provided (they fault as unsupported instructions): `fldenv`,
//! `fstenv`, `fsave`, `frstor`, `fbld`, `fbstp`.

use super::*;
use crate::fault::Fault;
use crate::mem::Memory;

/// A memory operand's format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FMem {
    F32,
    F64,
    F80,
    I16,
    I32,
    I64,
}

/// `dst = dst op src`; the `r` forms reverse the operands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FArith {
    Add,
    Mul,
    Sub,
    Subr,
    Div,
    Divr,
}

/// A source: a memory operand or `st(i)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FSrc {
    St(u8),
    Mem(FMem, MemRef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FConst {
    One,
    L2t,
    L2e,
    Pi,
    Lg2,
    Ln2,
    Zero,
}

/// Operations on `st(0)` in place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FUnary {
    Chs,
    Abs,
    Sqrt,
    Rndint,
    F2xm1,
    Sin,
    Cos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FpuOp {
    /// `fld`/`fild` from memory, `fld st(i)`, `fld1` ...
    Load(FSrc),
    LoadConst(FConst),
    /// `fst`/`fstp`/`fist`/`fistp`/`fisttp` (`trunc`) to memory
    Store {
        to: FMem,
        mem: MemRef,
        pop: bool,
        trunc: bool,
    },
    /// `fst st(i)` / `fstp st(i)`
    StoreSt {
        i: u8,
        pop: bool,
    },
    /// `st(dst) = st(dst) op src`, then pop if asked
    Arith {
        op: FArith,
        dst: u8,
        src: FSrc,
        pop: bool,
    },
    /// `fcom`/`fucom` (and `p`, `pp` forms): `st(0)` against `src`
    Compare {
        src: FSrc,
        pops: u8,
        unordered: bool,
    },
    /// `fcomi`/`fucomi` (and `p`): the result goes to EFLAGS
    Comi {
        i: u8,
        pop: bool,
        unordered: bool,
    },
    /// `fcmovcc st(0), st(i)`; `cc` is the integer condition code
    Cmov {
        cc: u8,
        i: u8,
    },
    Tst,
    Xam,
    Xch(u8),
    Free(u8),
    Unary(FUnary),
    Yl2x,
    Yl2xp1,
    Ptan,
    Patan,
    Xtract,
    Prem,
    Prem1,
    Scale,
    Sincos,
    Decstp,
    Incstp,
    Ldcw(MemRef),
    Stcw(MemRef),
    /// `fnstsw m16`, or `fnstsw ax` (`None`)
    Stsw(Option<MemRef>),
    Init,
    Clex,
    Nop,
}

// status word bits
const IE: u16 = 0x0001;
const ZE: u16 = 0x0004;
const SF: u16 = 0x0040;
const C0: u16 = 0x0100;
const C1: u16 = 0x0200;
const C2: u16 = 0x0400;
const C3: u16 = 0x4000;
const CC: u16 = C0 | C1 | C2 | C3;
const TOP_SHIFT: u16 = 11;

/// The value an invalid operation produces with exceptions masked.
const INDEFINITE: f64 = -f64::NAN; // 0xfff8_0000_0000_0000
/// `fninit`'s control word: all exceptions masked, extended precision,
/// round to nearest.
const CW_INIT: u16 = 0x037f;

/// The x87 state.
#[derive(Clone, Debug)]
pub struct Fpu {
    /// Physical registers R0..R7; `st(i)` is `regs[(top + i) & 7]`.
    pub regs: [f64; 8],
    pub top: u8,
    /// Bit `n` set: `regs[n]` holds a value (else its tag is "empty").
    pub valid: u8,
    pub cw: u16,
    /// Status word without the `TOP` field (see [`Fpu::status`]).
    pub sw: u16,
}

impl Default for Fpu {
    fn default() -> Fpu {
        Fpu {
            regs: [0.0; 8],
            top: 0,
            valid: 0,
            cw: CW_INIT,
            sw: 0,
        }
    }
}

impl PartialEq for Fpu {
    fn eq(&self, o: &Fpu) -> bool {
        self.regs.map(f64::to_bits) == o.regs.map(f64::to_bits)
            && (self.top, self.valid, self.cw, self.sw) == (o.top, o.valid, o.cw, o.sw)
    }
}

impl Eq for Fpu {}

impl Fpu {
    fn phys(&self, i: u8) -> usize {
        ((self.top + i) & 7) as usize
    }

    pub fn is_empty(&self, i: u8) -> bool {
        self.valid & (1 << self.phys(i)) == 0
    }

    /// `st(i)`; an empty register is a stack underflow: invalid operation,
    /// and the indefinite NaN as the value.
    pub fn st(&mut self, i: u8) -> f64 {
        if self.is_empty(i) {
            self.sw = (self.sw & !C1) | IE | SF;
            return INDEFINITE;
        }
        self.regs[self.phys(i)]
    }

    pub fn set(&mut self, i: u8, v: f64) {
        let p = self.phys(i);
        self.regs[p] = v;
        self.valid |= 1 << p;
    }

    /// Push; onto a full stack this is an overflow (the value becomes the
    /// indefinite NaN), as the x87 does with exceptions masked.
    pub fn push(&mut self, v: f64) {
        self.top = (self.top + 7) & 7;
        if !self.is_empty(0) {
            self.sw |= IE | SF | C1;
            self.set(0, INDEFINITE);
        } else {
            self.set(0, v);
        }
    }

    pub fn pop(&mut self) {
        self.valid &= !(1 << self.phys(0));
        self.top = (self.top + 1) & 7;
    }

    /// The status word as `fnstsw` stores it.
    pub fn status(&self) -> u16 {
        (self.sw & !(7 << TOP_SHIFT)) | (u16::from(self.top) << TOP_SHIFT)
    }

    fn set_cc(&mut self, c3: bool, c2: bool, c1: bool, c0: bool) {
        self.sw &= !CC;
        for (on, bit) in [(c3, C3), (c2, C2), (c1, C1), (c0, C0)] {
            if on {
                self.sw |= bit;
            }
        }
    }

    /// Round to an integer as the control word's RC field says.
    fn round(&self, v: f64) -> f64 {
        match (self.cw >> 10) & 3 {
            0 => v.round_ties_even(),
            1 => v.floor(),
            2 => v.ceil(),
            _ => v.trunc(),
        }
    }

    fn reset(&mut self) {
        *self = Fpu::default();
    }
}

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
fn shr_round(m: u64, sh: u32) -> u64 {
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
    // value = m * 2^(e' - 16383 - 63), e' = 1 for denormals
    let lz = m.leading_zeros();
    let m = m << lz;
    let mut exp2 = e.max(1) - 16383 - lz as i32; // m now has its top bit set
    if exp2 >= -1022 {
        let mut q = shr_round(m, 11);
        if q == 1 << 53 {
            q >>= 1;
            exp2 += 1;
        }
        if exp2 > 1023 {
            return f64::from_bits(sign | (0x7ff << 52));
        }
        f64::from_bits(sign | (((exp2 + 1023) as u64) << 52) | (q & ((1 << 52) - 1)))
    } else {
        // subnormal in f64 (a carry into bit 52 is the smallest normal)
        let sh = (-1011 - exp2) as u32;
        f64::from_bits(sign | shr_round(m, sh))
    }
}

/// `v * 2^n` without overflowing the intermediate power.
fn ldexp(mut v: f64, mut n: i64) -> f64 {
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
fn split(v: f64) -> (f64, i64) {
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
fn compare(a: f64, b: f64) -> (bool, bool, bool) {
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

/// Inputs at or beyond 2^63 are out of range for `fsin` and friends: C2
/// is set and the operand is left alone.
const TRIG_LIMIT: f64 = (1u64 << 63) as f64;

impl Cpu {
    fn fload(&mut self, mem: &Memory, src: FSrc) -> Result<f64, Fault> {
        let FSrc::Mem(fmt, m) = src else {
            let FSrc::St(i) = src else { unreachable!() };
            return Ok(self.fpu.st(i));
        };
        let a = self.ea(&m);
        Ok(match fmt {
            FMem::F32 => f64::from(f32::from_bits(mem.read_u32(a)?)),
            FMem::F64 => f64::from_bits(read_u64(mem, a)?),
            FMem::F80 => f80_to_f64(mem.slice(a, 10)?.try_into().unwrap()),
            FMem::I16 => f64::from(mem.read_u16(a)? as i16),
            FMem::I32 => f64::from(mem.read_u32(a)? as i32),
            FMem::I64 => read_u64(mem, a)? as i64 as f64,
        })
    }

    fn fstore(&mut self, mem: &mut Memory, to: FMem, m: MemRef, v: f64, trunc: bool) -> R {
        let a = self.ea(&m);
        let int = |cpu: &mut Cpu, lo: f64, hi: f64| -> Option<f64> {
            let r = if trunc { v.trunc() } else { cpu.fpu.round(v) };
            if r >= lo && r < hi {
                Some(r)
            } else {
                cpu.fpu.sw |= IE;
                None
            }
        };
        match to {
            FMem::F32 => mem.write_u32(a, (v as f32).to_bits()),
            FMem::F64 => mem.write_bytes(a, &v.to_bits().to_le_bytes()),
            FMem::F80 => mem.write_bytes(a, &f64_to_f80(v)),
            FMem::I16 => {
                let r = int(self, -32768.0, 32768.0).map_or(0x8000, |r| r as i16 as u16);
                mem.write_u16(a, r)
            }
            FMem::I32 => {
                let r =
                    int(self, -2147483648.0, 2147483648.0).map_or(0x8000_0000, |r| r as i32 as u32);
                mem.write_u32(a, r)
            }
            FMem::I64 => {
                let r = int(self, -TRIG_LIMIT, TRIG_LIMIT).map_or(1 << 63, |r| r as i64 as u64);
                mem.write_bytes(a, &r.to_le_bytes())
            }
        }
    }

    /// Execute one x87 instruction.
    pub(crate) fn fpu_exec(&mut self, mem: &mut Memory, op: FpuOp) -> R {
        let f = &mut self.fpu;
        match op {
            FpuOp::Load(src) => {
                let v = self.fload(mem, src)?;
                self.fpu.push(v);
            }
            FpuOp::LoadConst(c) => f.push(match c {
                FConst::One => 1.0,
                FConst::L2t => std::f64::consts::LOG2_10,
                FConst::L2e => std::f64::consts::LOG2_E,
                FConst::Pi => std::f64::consts::PI,
                FConst::Lg2 => std::f64::consts::LOG10_2,
                FConst::Ln2 => std::f64::consts::LN_2,
                FConst::Zero => 0.0,
            }),
            FpuOp::Store {
                to,
                mem: m,
                pop,
                trunc,
            } => {
                let v = self.fpu.st(0);
                self.fstore(mem, to, m, v, trunc)?;
                if pop {
                    self.fpu.pop();
                }
            }
            FpuOp::StoreSt { i, pop } => {
                let v = f.st(0);
                f.set(i, v);
                if pop {
                    f.pop();
                }
            }
            FpuOp::Arith { op, dst, src, pop } => {
                let s = self.fload(mem, src)?;
                let f = &mut self.fpu;
                let d = f.st(dst);
                let (num, den) = match op {
                    FArith::Div => (d, s),
                    FArith::Divr => (s, d),
                    _ => (1.0, 1.0),
                };
                if den == 0.0 && num.is_finite() && num != 0.0 {
                    f.sw |= ZE;
                }
                let r = match op {
                    FArith::Add => d + s,
                    FArith::Mul => d * s,
                    FArith::Sub => d - s,
                    FArith::Subr => s - d,
                    FArith::Div | FArith::Divr => num / den,
                };
                if r.is_nan() && !d.is_nan() && !s.is_nan() {
                    f.sw |= IE;
                }
                f.set(dst, r);
                if pop {
                    f.pop();
                }
            }
            FpuOp::Compare {
                src,
                pops,
                unordered,
            } => {
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
            }
            FpuOp::Comi { i, pop, unordered } => {
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
            FpuOp::Cmov { cc, i } => {
                if self.cond(cc) {
                    let f = &mut self.fpu;
                    let v = f.st(i);
                    f.set(0, v);
                }
            }
            FpuOp::Tst => {
                let a = f.st(0);
                let (c3, c2, c0) = compare(a, 0.0);
                f.set_cc(c3, c2, false, c0);
            }
            FpuOp::Xam => {
                let p = f.phys(0);
                let v = f.regs[p];
                let sign = v.is_sign_negative();
                let (c3, c2, c0) = if f.is_empty(0) {
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
                };
                f.set_cc(c3, c2, sign && !f.is_empty(0), c0);
            }
            FpuOp::Xch(i) => {
                let (a, b) = (f.st(0), f.st(i));
                f.set(0, b);
                f.set(i, a);
                f.sw &= !C1;
            }
            FpuOp::Free(i) => {
                let p = f.phys(i);
                f.valid &= !(1 << p);
            }
            FpuOp::Unary(u) => {
                let v = f.st(0);
                if matches!(u, FUnary::Sin | FUnary::Cos) && v.abs() >= TRIG_LIMIT {
                    f.sw |= C2;
                    return Ok(());
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
                if matches!(u, FUnary::Sin | FUnary::Cos) {
                    f.sw &= !C2;
                }
                f.set(0, r);
            }
            FpuOp::Yl2x | FpuOp::Yl2xp1 => {
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
            FpuOp::Ptan | FpuOp::Sincos => {
                let v = f.st(0);
                if v.abs() >= TRIG_LIMIT {
                    f.sw |= C2;
                    return Ok(());
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
            FpuOp::Patan => {
                let (x, y) = (f.st(0), f.st(1));
                f.set(1, y.atan2(x));
                f.pop();
            }
            FpuOp::Xtract => {
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
            FpuOp::Prem | FpuOp::Prem1 => {
                let (a, b) = (f.st(0), f.st(1));
                if b == 0.0 || !a.is_finite() || b.is_nan() {
                    f.sw |= IE;
                    f.set(0, INDEFINITE);
                    f.set_cc(false, false, false, false);
                    return Ok(());
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
            FpuOp::Scale => {
                let (v, s) = (f.st(0), f.st(1));
                let r = if s.is_nan() {
                    s
                } else {
                    ldexp(v, s.trunc().clamp(-1e5, 1e5) as i64)
                };
                f.set(0, r);
            }
            FpuOp::Decstp => f.top = (f.top + 7) & 7,
            FpuOp::Incstp => f.top = (f.top + 1) & 7,
            FpuOp::Ldcw(m) => self.fpu.cw = mem.read_u16(self.ea(&m))?,
            FpuOp::Stcw(m) => mem.write_u16(self.ea(&m), self.fpu.cw)?,
            FpuOp::Stsw(None) => {
                let s = self.fpu.status();
                self.set_reg(EAX, Size::W, u32::from(s));
            }
            FpuOp::Stsw(Some(m)) => mem.write_u16(self.ea(&m), self.fpu.status())?,
            FpuOp::Init => f.reset(),
            FpuOp::Clex => f.sw &= !0x80ff,
            FpuOp::Nop => {}
        }
        Ok(())
    }
}

type R = Result<(), Fault>;

fn read_u64(mem: &Memory, a: u32) -> Result<u64, Fault> {
    Ok(u64::from_le_bytes(mem.slice(a, 8)?.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f80_round_trips_every_kind_of_double() {
        for v in [
            0.0,
            -0.0,
            1.0,
            -2.5,
            std::f64::consts::PI,
            f64::MAX,
            f64::MIN_POSITIVE,
            5e-324,
            1e-310,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert_eq!(f80_to_f64(&f64_to_f80(v)).to_bits(), v.to_bits(), "{v}");
        }
        assert!(f80_to_f64(&f64_to_f80(f64::NAN)).is_nan());
    }

    #[test]
    fn f80_bytes_match_the_x87() {
        // 1.0 and pi as `fstp tbyte` writes them
        assert_eq!(f64_to_f80(1.0), [0, 0, 0, 0, 0, 0, 0, 0x80, 0xff, 0x3f]);
        let pi80 = [0x35, 0xc2, 0x68, 0x21, 0xa2, 0xda, 0x0f, 0xc9, 0x00, 0x40];
        assert_eq!(f80_to_f64(&pi80), std::f64::consts::PI);
        // more precision than a double holds: rounded to nearest
        let third = [0xab, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xaa, 0xfd, 0x3f];
        assert_eq!(f80_to_f64(&third), 1.0 / 3.0);
        // beyond the double range
        assert_eq!(
            f80_to_f64(&[0, 0, 0, 0, 0, 0, 0, 0x80, 0x00, 0x44]),
            f64::INFINITY
        );
        assert_eq!(f80_to_f64(&[0, 0, 0, 0, 0, 0, 0, 0x80, 0x01, 0x00]), 0.0);
    }

    #[test]
    fn stack_tags_overflow_and_underflow() {
        let mut f = Fpu::default();
        assert!(f.st(0).is_nan());
        assert_eq!(f.status() & (IE | SF), IE | SF);
        f.reset();
        for i in 0..8 {
            f.push(f64::from(i));
        }
        assert_eq!(f.st(0), 7.0);
        assert_eq!(f.status() & IE, 0);
        f.push(8.0);
        assert!(f.st(0).is_nan());
        assert_eq!(f.status() & (IE | SF | C1), IE | SF | C1);
    }

    /// Run x87 code at 0x2000 (data at 0x4000) and return the CPU and memory.
    fn run(code: &[u8], data: &[u8]) -> (Cpu, Memory) {
        let mut m = Memory::new(0x8000);
        m.write_bytes(0x2000, code).unwrap();
        m.write_bytes(0x4000, data).unwrap();
        let mut c = Cpu {
            esp_floor: 0x6000,
            ..Cpu::default()
        };
        c.r[EBX as usize] = 0x4000;
        c.eip = 0x2000;
        while c.eip < 0x2000 + code.len() as u32 {
            let i = crate::x86::decode(&m, c.eip).unwrap();
            c.exec(&mut m, &i).unwrap();
        }
        (c, m)
    }

    #[test]
    fn floats_library_idioms() {
        // fld1 fld1 faddp (f+ is `de c1`), fldpi, fdivp: st0 = 2/pi
        let (mut c, _) = run(
            &[0xd9, 0xe8, 0xd9, 0xe8, 0xde, 0xc1, 0xd9, 0xeb, 0xde, 0xf9],
            &[],
        );
        assert_eq!(c.fpu.st(0), 2.0 / std::f64::consts::PI);
        assert_eq!(c.fpu.top, 7);
        // f- is `de e9`: a b -- a-b
        let mut data = 10.0f64.to_le_bytes().to_vec();
        data.extend(4.0f64.to_le_bytes());
        let (mut c, _) = run(&[0xdd, 0x03, 0xdd, 0x43, 0x08, 0xde, 0xe9], &data);
        assert_eq!(c.fpu.st(0), 6.0);
        // fstp tbyte [ebx+16], fld tbyte [ebx+16]: 80-bit round trip
        let (mut c, m) = run(&[0xd9, 0xeb, 0xdb, 0x7b, 0x10, 0xdb, 0x6b, 0x10], &[]);
        assert_eq!(c.fpu.st(0), std::f64::consts::PI);
        assert_eq!(
            m.slice(0x4010, 10).unwrap(),
            &f64_to_f80(std::f64::consts::PI)
        );
    }

    #[test]
    fn integer_conversion_follows_the_rounding_control() {
        // fild dword [ebx]; fld1; fadd st0,st0 (2); fdivp (7/2 = 3.5); fist [ebx+4]
        let code = [
            0xdb, 0x03, 0xd9, 0xe8, 0xd8, 0xc0, 0xde, 0xf9, 0xdb, 0x53, 0x04,
        ];
        let (_, m) = run(&code, &7u32.to_le_bytes());
        assert_eq!(m.read_u32(0x4004).unwrap(), 4); // nearest-even: 3.5 -> 4
                                                    // the same after `fldcw` with RC = truncate (floats' `finit`)
        let mut data = 7u32.to_le_bytes().to_vec();
        data.extend([0, 0, 0, 0]);
        data.extend(0x0f3fu16.to_le_bytes());
        let mut code2 = vec![0xd9, 0x6b, 0x08];
        code2.extend(code);
        let (_, m) = run(&code2, &data);
        assert_eq!(m.read_u32(0x4004).unwrap(), 3);
        // out of range: the integer indefinite
        let (_, m) = run(&[0xd9, 0xe8, 0xd9, 0xee, 0xde, 0xf9, 0xdb, 0x1b], &[]);
        assert_eq!(m.read_u32(0x4000).unwrap(), 0x8000_0000);
    }

    #[test]
    fn comparisons_reach_the_status_word_and_eflags() {
        // fld1 fldz fcompp fnstsw ax: 0 < 1 -> C0 (bit 8)
        let (c, _) = run(&[0xd9, 0xe8, 0xd9, 0xee, 0xde, 0xd9, 0xdf, 0xe0], &[]);
        assert_eq!(c.r[EAX as usize] & 0x4500, 0x0100);
        assert_eq!(c.fpu.valid, 0);
        // fldz fld1 fcomip st1: 1 > 0 -> CF = ZF = 0
        let (c, _) = run(&[0xd9, 0xee, 0xd9, 0xe8, 0xdf, 0xf1], &[]);
        assert!(!c.cf && !c.zf && !c.pf);
        // fxam on an empty register: C3 and C0
        let (c, _) = run(&[0xd9, 0xe5, 0xdf, 0xe0], &[]);
        assert_eq!(c.r[EAX as usize] & 0x4700, 0x4100);
    }

    #[test]
    fn transcendentals() {
        // fldln2 fld1 fadd st0,st0 ... fyl2x: ln(2) * log2(2) = ln 2;  fsin of pi/2
        let (mut c, _) = run(
            &[0xd9, 0xe8, 0xd8, 0xc0, 0xd9, 0xed, 0xd9, 0xc9, 0xd9, 0xf1],
            &[],
        );
        assert_eq!(c.fpu.st(0), std::f64::consts::LN_2);
        let (mut c, _) = run(
            &[0xd9, 0xeb, 0xd9, 0xe8, 0xd8, 0xc0, 0xde, 0xf9, 0xd9, 0xfe],
            &[],
        );
        assert_eq!(c.fpu.st(0), 1.0);
        // fxtract of 12: exponent 3 below significand 1.5
        let mut data = 12.0f64.to_le_bytes().to_vec();
        data.extend([0; 8]);
        let (mut c, _) = run(&[0xdd, 0x03, 0xd9, 0xf4], &data);
        assert_eq!((c.fpu.st(0), c.fpu.st(1)), (1.5, 3.0));
    }

    #[test]
    fn indefinite_is_the_x87_bit_pattern() {
        assert_eq!(INDEFINITE.to_bits(), 0xfff8_0000_0000_0000);
    }

    #[test]
    fn helpers() {
        assert_eq!(split(12.0), (1.5, 3));
        assert_eq!(split(5e-324), (1.0, -1074));
        assert_eq!(ldexp(1.0, 1030), f64::INFINITY);
        assert_eq!(ldexp(1.0, -1074), 5e-324);
        assert_eq!(ldexp(3.0, 4), 48.0);
    }
}
