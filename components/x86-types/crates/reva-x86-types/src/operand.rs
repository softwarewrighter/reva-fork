//! Operands: sizes, memory references, registers and immediates.

/// Operand size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Size {
    B,
    W,
    D,
}

impl Size {
    #[inline]
    pub fn bytes(self) -> u32 {
        match self {
            Size::B => 1,
            Size::W => 2,
            Size::D => 4,
        }
    }
    #[inline]
    pub fn mask(self) -> u32 {
        match self {
            Size::B => 0xff,
            Size::W => 0xffff,
            Size::D => 0xffff_ffff,
        }
    }
    #[inline]
    pub fn sign(self) -> u32 {
        match self {
            Size::B => 0x80,
            Size::W => 0x8000,
            Size::D => 0x8000_0000,
        }
    }
}

/// `[base + index*scale + disp]`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemRef {
    pub base: Option<u8>,
    pub index: Option<u8>,
    pub scale: u8,
    pub disp: u32,
}

/// An operand.  For [`Size::B`] a `Reg(n)` names AL CL DL BL AH CH DH BH.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opd {
    Reg(u8),
    Mem(MemRef),
    Imm(u32),
}
