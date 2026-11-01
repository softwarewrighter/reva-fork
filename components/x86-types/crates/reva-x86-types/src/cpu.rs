//! The CPU: registers, EIP, flags, the x87.
//!
//! Register roles are Reva's, unchanged:
//!
//! | register | role                                     |
//! |----------|------------------------------------------|
//! | EAX      | top of data stack (TOS)                  |
//! | ESI      | data stack pointer; `[ESI]` is NOS       |
//! | ESP      | return stack pointer (in VM memory)      |
//! | EBX ECX EDX EDI EBP | scratch                       |

use reva_x86_fpustate::fpu::Fpu;

pub const EAX: u8 = 0;
pub const ECX: u8 = 1;
pub const EDX: u8 = 2;
pub const EBX: u8 = 3;
pub const ESP: u8 = 4;
pub const EBP: u8 = 5;
pub const ESI: u8 = 6;
pub const EDI: u8 = 7;

pub const REG_NAMES: [&str; 8] = ["eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi"];

/// The CPU state: eight registers, EIP, and the flags Reva code observes.
/// (AF is not modelled; nothing in Reva uses BCD arithmetic.)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cpu {
    pub r: [u32; 8],
    pub eip: u32,
    pub cf: bool,
    pub zf: bool,
    pub sf: bool,
    pub of: bool,
    pub pf: bool,
    pub df: bool,
    /// Lowest legal ESP; pushes below it fault.
    pub esp_floor: u32,
    pub fpu: Fpu,
}

impl Cpu {
    /// Evaluate condition code `cc` (the low nibble of `Jcc`/`SETcc`).
    #[inline]
    pub fn cond(&self, cc: u8) -> bool {
        let r = match cc >> 1 {
            0 => self.of,
            1 => self.cf,
            2 => self.zf,
            3 => self.cf || self.zf,
            4 => self.sf,
            5 => self.pf,
            6 => self.sf != self.of,
            _ => self.zf || (self.sf != self.of),
        };
        r ^ (cc & 1 == 1)
    }
}
