//! `exec`: one instruction, by group.

use crate::control::MiscOps;
use crate::data::DataOps;
use reva_x86_base::fault::Fault;
use reva_x86_fpuexec::dispatch::FpuExec;
use reva_x86_mem::memory::Memory;
use reva_x86_ops::alu::Arith;
use reva_x86_ops::muldiv::MulDivs;
use reva_x86_ops::operands::Operands;
use reva_x86_ops::regs::Regs;
use reva_x86_shift::string::Strings;
use reva_x86_types::cpu::{Cpu, ESP};
use reva_x86_types::op::{Alu, Insn, Op};
use reva_x86_types::operand::Size;

type R = Result<(), Fault>;

reva_macros::ext_trait! {
    impl Cpu;
    /// Executing instructions.
    pub trait Exec: DataOps, MiscOps, MulDivs, Strings, FpuExec {
        /// Execute `insn`, which was decoded at `self.eip`.
        #[inline(always)]
        fn exec(&mut self, mem: &mut Memory, insn: &Insn) -> R {
            let at = self.eip;
            let next = at.wrapping_add(insn.len as u32);
            self.eip = next;
            match insn.op {
                op @ (Op::Alu { .. } | Op::Test { .. } | Op::Mov { .. } | Op::Movx { .. }
                | Op::Lea { .. } | Op::Xchg { .. } | Op::Push(_) | Op::Pop(_)) => self.exec_data(mem, op),
                op @ (Op::Inc { .. } | Op::Dec { .. } | Op::Not { .. } | Op::Neg { .. }
                | Op::Shift { .. } | Op::MulDiv { .. } | Op::Imul { .. }) => self.exec_arith(mem, op, at),
                op @ (Op::Cdq | Op::Cwde | Op::Cbw | Op::Cwd | Op::Bswap(_) | Op::Str { .. }
                | Op::Setcc { .. } | Op::Cmovcc { .. } | Op::Flag(_) | Op::Nop | Op::Fpu(_)) => {
                    self.exec_misc(mem, op)
                }
                op => self.exec_flow(mem, op, at, next),
            }
        }

        /// Moves, the ALU, `test`, `lea`, `xchg`, `push`, `pop`.
        #[inline(always)]
        fn exec_data(&mut self, mem: &mut Memory, op: Op) -> R {
            match op {
                Op::Alu { op, size, dst, src } => self.alu_insn(mem, op, size, dst, src),
                Op::Test { size, a, b } => {
                    let (x, y) = (self.read(mem, a, size)?, self.read(mem, b, size)?);
                    self.alu(Alu::And, size, x, y);
                    Ok(())
                }
                Op::Mov { size, dst, src } => {
                    let v = self.read(mem, src, size)?;
                    self.write(mem, dst, size, v)
                }
                Op::Movx { signed, from, dst, src } => self.movx(mem, signed, from, dst, src),
                Op::Lea { dst, mem: m } => {
                    self.r[dst as usize] = self.ea(&m);
                    Ok(())
                }
                Op::Xchg { size, a, b } => self.xchg(mem, size, a, b),
                Op::Push(o) => {
                    let v = self.read(mem, o, Size::D)?;
                    self.push32(mem, v)
                }
                Op::Pop(o) => {
                    let v = self.pop32(mem)?;
                    self.write(mem, o, Size::D, v)
                }
                _ => unreachable!("exec_data: {op:?}"),
            }
        }

        /// `inc`, `dec`, `not`, `neg`, shifts, `mul`/`div`, `imul`.
        #[inline(always)]
        fn exec_arith(&mut self, mem: &mut Memory, op: Op, at: u32) -> R {
            match op {
                Op::Inc { size, dst } => self.inc_dec(mem, Alu::Add, size, dst),
                Op::Dec { size, dst } => self.inc_dec(mem, Alu::Sub, size, dst),
                Op::Not { size, dst } => {
                    let v = self.read(mem, dst, size)?;
                    self.write(mem, dst, size, !v)
                }
                Op::Neg { size, dst } => {
                    let v = self.read(mem, dst, size)?;
                    let r = self.alu(Alu::Sub, size, 0, v);
                    self.write(mem, dst, size, r)
                }
                Op::Shift { op, size, dst, count } => self.shift_insn(mem, op, size, dst, count),
                Op::MulDiv { op, size, src } => {
                    let s = self.read(mem, src, size)?;
                    self.muldiv(op, size, s, at)
                }
                Op::Imul { dst, src, imm } => self.imul(mem, dst, src, imm),
                _ => unreachable!("exec_arith: {op:?}"),
            }
        }

        /// Conversions, `bswap`, strings, `setcc`, `cmovcc`, flags, `nop`, x87.
        fn exec_misc(&mut self, mem: &mut Memory, op: Op) -> R {
            match op {
                Op::Bswap(r) => self.r[r as usize] = self.r[r as usize].swap_bytes(),
                Op::Str { op, size, rep } => return self.rep_string(mem, op, size, rep),
                Op::Setcc { cc, dst } => {
                    let v = self.cond(cc) as u32;
                    return self.write(mem, dst, Size::B, v);
                }
                Op::Cmovcc { cc, dst, src } => return self.cmovcc(mem, cc, dst, src),
                Op::Flag(f) => self.flag(f),
                Op::Nop => {}
                Op::Fpu(f) => return self.fpu_exec(mem, f),
                _ => self.convert(op), // Cdq, Cwde, Cbw, Cwd
            }
            Ok(())
        }

        /// Branches, calls, returns, loops, traps.
        #[inline(always)]
        fn exec_flow(&mut self, mem: &mut Memory, op: Op, at: u32, next: u32) -> R {
            match op {
                Op::Jcc { cc, target } => {
                    if self.cond(cc) {
                        self.eip = target;
                    }
                }
                Op::Jmp(t) => self.eip = t,
                Op::JmpInd(o) => self.eip = self.read(mem, o, Size::D)?,
                Op::Call(t) => {
                    self.push32(mem, next)?;
                    self.eip = t;
                }
                Op::CallInd(o) => {
                    let t = self.read(mem, o, Size::D)?;
                    self.push32(mem, next)?;
                    self.eip = t;
                }
                Op::Ret(n) => {
                    self.eip = self.pop32(mem)?;
                    self.r[ESP as usize] = self.r[ESP as usize].wrapping_add(n as u32);
                }
                Op::Loop { kind, target } => self.loop_insn(kind, target),
                Op::Hlt => return Err(Fault::Trap { eip: at, what: "hlt" }),
                _ => return Err(Fault::Trap { eip: at, what: "int3 breakpoint" }), // Int3
            }
            Ok(())
        }
    }
}
