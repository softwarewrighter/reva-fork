//! # reva-x86-fpuexec — executing x87 instructions
//!
//! `lib/math/floats` (and CoSy, which keeps its floats as 80-bit values)
//! writes x87 instructions directly as bytes: `f+` is `inline{ de c1 }`,
//! `f@` is `fld tbyte [eax]`.  This crate runs them on reva-x86-types'
//! `Cpu` (whose `fpu` is reva-x86-fpustate's register stack).
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

pub mod arith;
pub mod dispatch;
pub mod math;
pub mod memory;
