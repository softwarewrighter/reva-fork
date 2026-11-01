//! # reva-x86-float — pure x87 arithmetic
//!
//! What the x87 executor computes without touching the machine: the
//! 80-bit memory format ([`f80`]), scaling, splitting, comparison and
//! classification ([`math`]), constants and the arithmetic forms
//! ([`values`]).  Registers hold `f64`: loads and stores of the 80-bit
//! format are exact in the direction f64 → f80 and round to nearest-even
//! in the direction f80 → f64.

pub mod f80;
pub mod math;
pub mod values;
