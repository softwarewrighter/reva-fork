//! One x87 instruction, by group.

use crate::arith::FpuArith;
use crate::math::FpuMath;
use crate::memory::FpuMemory;
use reva_x86_base::fault::Fault;
use reva_x86_float::math::{compare, scale_value};
use reva_x86_float::values::constant;
use reva_x86_fpustate::fpu::C1;
use reva_x86_mem::memory::Memory;
use reva_x86_ops::regs::Regs;
use reva_x86_types::cpu::{Cpu, EAX};
use reva_x86_types::fpu_op::FpuOp;
use reva_x86_types::operand::Size;

type R = Result<(), Fault>;

reva_macros::ext_trait! {
    impl Cpu;
    /// Executing x87 instructions.
    pub trait FpuExec: FpuMemory, FpuArith, FpuMath, Regs {
        /// Execute one x87 instruction.
        fn fpu_exec(&mut self, mem: &mut Memory, op: FpuOp) -> R {
            match op {
                FpuOp::Load(_) | FpuOp::LoadConst(_) | FpuOp::Store { .. } | FpuOp::StoreSt { .. }
                | FpuOp::Xch(_) | FpuOp::Free(_) | FpuOp::Decstp | FpuOp::Incstp => {
                    self.fpu_stack(mem, op)
                }
                FpuOp::Ldcw(_) | FpuOp::Stcw(_) | FpuOp::Stsw(_) | FpuOp::Init | FpuOp::Clex
                | FpuOp::Nop => self.fpu_control(mem, op),
                FpuOp::Arith { .. } | FpuOp::Compare { .. } | FpuOp::Comi { .. }
                | FpuOp::Cmov { .. } | FpuOp::Tst | FpuOp::Xam => self.fpu_compare(mem, op),
                _ => {
                    self.fpu_math(op);
                    Ok(())
                }
            }
        }

        /// Loads, stores and register-stack moves.
        fn fpu_stack(&mut self, mem: &mut Memory, op: FpuOp) -> R {
            let f = &mut self.fpu;
            match op {
                FpuOp::Load(src) => {
                    let v = self.fload(mem, src)?;
                    self.fpu.push(v);
                }
                FpuOp::LoadConst(c) => f.push(constant(c)),
                FpuOp::Store { to, mem: m, pop, trunc } => self.store_st0(mem, to, m, pop, trunc)?,
                FpuOp::StoreSt { i, pop } => {
                    let v = f.st(0);
                    f.set(i, v);
                    if pop {
                        f.pop();
                    }
                }
                FpuOp::Xch(i) => {
                    let (a, b) = (f.st(0), f.st(i));
                    f.set(0, b);
                    f.set(i, a);
                    f.sw &= !C1;
                }
                FpuOp::Free(i) => f.valid &= !(1 << f.phys(i)),
                FpuOp::Decstp => f.top = (f.top + 7) & 7,
                _ => f.top = (f.top + 1) & 7, // Incstp
            }
            Ok(())
        }

        /// The control and status words, `fninit`, `fnclex`, `fnop`.
        fn fpu_control(&mut self, mem: &mut Memory, op: FpuOp) -> R {
            match op {
                FpuOp::Ldcw(m) => self.fpu.cw = mem.read_u16(self.ea(&m))?,
                FpuOp::Stcw(m) => mem.write_u16(self.ea(&m), self.fpu.cw)?,
                FpuOp::Stsw(None) => {
                    let s = self.fpu.status();
                    self.set_reg(EAX, Size::W, u32::from(s));
                }
                FpuOp::Stsw(Some(m)) => mem.write_u16(self.ea(&m), self.fpu.status())?,
                FpuOp::Init => self.fpu.reset(),
                FpuOp::Clex => self.fpu.sw &= !0x80ff,
                _ => {} // Nop
            }
            Ok(())
        }

        /// Arithmetic, comparisons, `fcmov`, `ftst`, `fxam`.
        fn fpu_compare(&mut self, mem: &mut Memory, op: FpuOp) -> R {
            match op {
                FpuOp::Arith { op, dst, src, pop } => self.arith(mem, op, dst, src, pop)?,
                FpuOp::Compare { src, pops, unordered } => self.compare_op(mem, src, pops, unordered)?,
                FpuOp::Comi { i, pop, unordered } => self.comi(i, pop, unordered),
                FpuOp::Cmov { cc, i } => {
                    if self.cond(cc) {
                        let v = self.fpu.st(i);
                        self.fpu.set(0, v);
                    }
                }
                FpuOp::Tst => {
                    let a = self.fpu.st(0);
                    let (c3, c2, c0) = compare(a, 0.0);
                    self.fpu.set_cc(c3, c2, false, c0);
                }
                _ => self.xam(),
            }
            Ok(())
        }

        /// The mathematical functions.
        fn fpu_math(&mut self, op: FpuOp) {
            let f = &mut self.fpu;
            match op {
                FpuOp::Unary(u) => self.unary(u),
                FpuOp::Yl2x | FpuOp::Yl2xp1 => self.log2(op),
                FpuOp::Ptan | FpuOp::Sincos => self.trig(op),
                FpuOp::Patan => {
                    let (x, y) = (f.st(0), f.st(1));
                    f.set(1, y.atan2(x));
                    f.pop();
                }
                FpuOp::Xtract => self.xtract(),
                FpuOp::Prem | FpuOp::Prem1 => self.prem(op),
                _ => {
                    // Scale
                    let (v, s) = (f.st(0), f.st(1));
                    f.set(0, scale_value(v, s));
                }
            }
        }
    }
}
