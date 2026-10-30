//! Shifts, sign conversions, `cmovcc`, `loop`, the flag instructions.

use reva_x86_base::fault::Fault;
use reva_x86_mem::memory::Memory;
use reva_x86_ops::operands::Operands;
use reva_x86_shift::shift::Shifts;
use reva_x86_types::cpu::{Cpu, ECX};
use reva_x86_types::op::{FlagOp, LoopKind, Op, Shift};
use reva_x86_types::operand::{Opd, Size};

type R = Result<(), Fault>;

reva_macros::ext_trait! {
    impl Cpu;
    /// Instructions on flags, counts and the accumulator.
    pub trait MiscOps: Operands, Shifts {
        /// A shift by `count & 31` (a zero count changes nothing).
        fn shift_insn(&mut self, mem: &mut Memory, op: Shift, size: Size, dst: Opd, count: Opd) -> R {
            let cnt = self.read(mem, count, Size::B)? & 0x1f;
            if cnt != 0 {
                let v = self.read(mem, dst, size)?;
                let r = self.shift(op, size, v, cnt);
                self.write(mem, dst, size, r)?;
            }
            Ok(())
        }

        /// `cdq`, `cwde`, `cbw`, `cwd`: sign extensions of the accumulator.
        fn convert(&mut self, op: Op) {
            match op {
                Op::Cdq => self.r[2] = if self.r[0] & 0x8000_0000 != 0 { u32::MAX } else { 0 },
                Op::Cwde => self.r[0] = self.r[0] as u16 as i16 as i32 as u32,
                Op::Cbw => {
                    self.r[0] = (self.r[0] & 0xffff_0000) | u32::from(self.r[0] as u8 as i8 as i16 as u16)
                }
                _ => {
                    // Cwd
                    let high = if self.r[0] & 0x8000 != 0 { 0xffff } else { 0 };
                    self.r[2] = (self.r[2] & 0xffff_0000) | high;
                }
            }
        }

        /// `cmovcc dst, src` (the source is read either way).
        fn cmovcc(&mut self, mem: &Memory, cc: u8, dst: u8, src: Opd) -> R {
            let v = self.read(mem, src, Size::D)?;
            if self.cond(cc) {
                self.r[dst as usize] = v;
            }
            Ok(())
        }

        /// `loop`, `loope`, `loopne`, `jecxz`.
        fn loop_insn(&mut self, kind: LoopKind, target: u32) {
            let c = &mut self.r[ECX as usize];
            let take = match kind {
                LoopKind::Jecxz => *c == 0,
                _ => {
                    *c = c.wrapping_sub(1);
                    *c != 0
                        && match kind {
                            LoopKind::Loope => self.zf,
                            LoopKind::Loopne => !self.zf,
                            _ => true,
                        }
                }
            };
            if take {
                self.eip = target;
            }
        }

        /// `clc`, `stc`, `cmc`, `cld`, `std`.
        fn flag(&mut self, f: FlagOp) {
            match f {
                FlagOp::Clc => self.cf = false,
                FlagOp::Stc => self.cf = true,
                FlagOp::Cmc => self.cf = !self.cf,
                FlagOp::Cld => self.df = false,
                FlagOp::Std => self.df = true,
            }
        }
    }
}
