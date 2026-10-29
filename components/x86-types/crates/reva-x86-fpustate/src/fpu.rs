//! The x87 state and its status-word bits.

// status word bits
pub const IE: u16 = 0x0001;
pub const ZE: u16 = 0x0004;
pub const SF: u16 = 0x0040;
pub const C0: u16 = 0x0100;
pub const C1: u16 = 0x0200;
pub const C2: u16 = 0x0400;
pub const C3: u16 = 0x4000;
pub const CC: u16 = C0 | C1 | C2 | C3;
pub const TOP_SHIFT: u16 = 11;

/// The value an invalid operation produces with exceptions masked.
pub const INDEFINITE: f64 = -f64::NAN; // 0xfff8_0000_0000_0000
/// `fninit`'s control word: all exceptions masked, extended precision,
/// round to nearest.
pub const CW_INIT: u16 = 0x037f;

/// The x87 state; its operations are in [`crate::stack`] and
/// [`crate::control`].
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
