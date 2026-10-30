//! The string instructions, once or with `rep`.

use reva_x86_base::fault::Fault;
use reva_x86_mem::memory::Memory;
use reva_x86_ops::alu::Arith;
use reva_x86_ops::regs::Regs;
use reva_x86_types::cpu::{Cpu, EAX, ECX, EDI, ESI};
use reva_x86_types::op::{Alu, Rep, StrOp};
use reva_x86_types::operand::Size;

type R = Result<(), Fault>;

/// A string element at `a`.
pub fn str_read(mem: &Memory, a: u32, size: Size) -> Result<u32, Fault> {
    match size {
        Size::B => mem.read_u8(a).map(u32::from),
        Size::W => mem.read_u16(a).map(u32::from),
        Size::D => mem.read_u32(a),
    }
}

/// Store a string element at `a`.
pub fn str_write(mem: &mut Memory, a: u32, size: Size, v: u32) -> R {
    match size {
        Size::B => mem.write_u8(a, v as u8),
        Size::W => mem.write_u16(a, v as u16),
        Size::D => mem.write_u32(a, v),
    }
}

reva_macros::ext_trait! {
    impl Cpu;
    /// String instructions.
    pub trait Strings: Regs, Arith {
        /// One step of `op`.
        fn string_once(&mut self, mem: &mut Memory, op: StrOp, size: Size) -> R {
            let (si, di) = (self.r[ESI as usize], self.r[EDI as usize]);
            match op {
                StrOp::Lods => {
                    let v = str_read(mem, si, size)?;
                    self.set_reg(EAX, size, v);
                }
                StrOp::Stos => str_write(mem, di, size, self.get_reg(EAX, size))?,
                StrOp::Movs => {
                    let v = str_read(mem, si, size)?;
                    str_write(mem, di, size, v)?;
                }
                StrOp::Scas => {
                    let b = str_read(mem, di, size)?;
                    self.alu(Alu::Cmp, size, self.get_reg(EAX, size), b);
                }
                StrOp::Cmps => {
                    let (a, b) = (str_read(mem, si, size)?, str_read(mem, di, size)?);
                    self.alu(Alu::Cmp, size, a, b);
                }
            }
            self.str_advance(op, size);
            Ok(())
        }

        /// Move ESI and/or EDI past the element, as DF says.
        fn str_advance(&mut self, op: StrOp, size: Size) {
            let n = size.bytes();
            let step = if self.df { n.wrapping_neg() } else { n };
            if matches!(op, StrOp::Lods | StrOp::Movs | StrOp::Cmps) {
                self.r[ESI as usize] = self.r[ESI as usize].wrapping_add(step);
            }
            if matches!(op, StrOp::Stos | StrOp::Movs | StrOp::Scas | StrOp::Cmps) {
                self.r[EDI as usize] = self.r[EDI as usize].wrapping_add(step);
            }
        }

        /// `op` once, or with `rep`/`repe`/`repne` while ECX is not 0.
        fn rep_string(&mut self, mem: &mut Memory, op: StrOp, size: Size, rep: Rep) -> R {
            if rep == Rep::None {
                return self.string_once(mem, op, size);
            }
            let cmp = matches!(op, StrOp::Scas | StrOp::Cmps);
            while self.r[ECX as usize] != 0 {
                self.string_once(mem, op, size)?;
                self.r[ECX as usize] -= 1;
                if cmp && ((rep == Rep::E && !self.zf) || (rep == Rep::Ne && self.zf)) {
                    break;
                }
            }
            Ok(())
        }
    }
}
