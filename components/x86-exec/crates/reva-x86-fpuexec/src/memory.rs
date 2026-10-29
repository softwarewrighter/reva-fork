//! Loads and stores between the x87 and memory.

use reva_x86_base::fault::Fault;
use reva_x86_float::f80::{f64_to_f80, f80_to_f64};
use reva_x86_float::values::TRIG_LIMIT;
use reva_x86_fpustate::fpu::IE;
use reva_x86_mem::memory::Memory;
use reva_x86_ops::regs::Regs;
use reva_x86_types::cpu::Cpu;
use reva_x86_types::fpu_op::{FMem, FSrc};
use reva_x86_types::operand::MemRef;

type R = Result<(), Fault>;

/// The little-endian 64-bit value at `a`.
pub fn read_u64(mem: &Memory, a: u32) -> Result<u64, Fault> {
    Ok(u64::from_le_bytes(mem.slice(a, 8)?.try_into().unwrap()))
}

reva_macros::ext_trait! {
    impl Cpu;
    /// The x87's memory operands.
    pub trait FpuMemory: Regs {
        /// The value of a source: `st(i)`, or memory in its format.
        fn fload(&mut self, mem: &Memory, src: FSrc) -> Result<f64, Fault> {
            let (fmt, m) = match src {
                FSrc::St(i) => return Ok(self.fpu.st(i)),
                FSrc::Mem(fmt, m) => (fmt, m),
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

        /// Store `v` to memory in format `to`; integers are rounded (or
        /// truncated) and out of range give the integer indefinite.
        fn fstore(&mut self, mem: &mut Memory, to: FMem, m: MemRef, v: f64, trunc: bool) -> R {
            let a = self.ea(&m);
            match to {
                FMem::F32 => mem.write_u32(a, (v as f32).to_bits()),
                FMem::F64 => mem.write_bytes(a, &v.to_bits().to_le_bytes()),
                FMem::F80 => mem.write_bytes(a, &f64_to_f80(v)),
                FMem::I16 => {
                    let r = self.fint(v, trunc, -32768.0, 32768.0).map_or(0x8000, |r| r as i16 as u16);
                    mem.write_u16(a, r)
                }
                FMem::I32 => {
                    let r = self
                        .fint(v, trunc, -2147483648.0, 2147483648.0)
                        .map_or(0x8000_0000, |r| r as i32 as u32);
                    mem.write_u32(a, r)
                }
                FMem::I64 => {
                    let r = self.fint(v, trunc, -TRIG_LIMIT, TRIG_LIMIT).map_or(1 << 63, |r| r as i64 as u64);
                    mem.write_bytes(a, &r.to_le_bytes())
                }
            }
        }

        /// `v` as an integer in `lo..hi` (rounded by the control word, or
        /// truncated); out of range is an invalid operation.
        fn fint(&mut self, v: f64, trunc: bool, lo: f64, hi: f64) -> Option<f64> {
            let r = if trunc { v.trunc() } else { self.fpu.round(v) };
            if r >= lo && r < hi {
                Some(r)
            } else {
                self.fpu.sw |= IE;
                None
            }
        }

        /// `fst`/`fstp`/`fist`... of st(0).
        fn store_st0(&mut self, mem: &mut Memory, to: FMem, m: MemRef, pop: bool, trunc: bool) -> R {
            let v = self.fpu.st(0);
            self.fstore(mem, to, m, v, trunc)?;
            if pop {
                self.fpu.pop();
            }
            Ok(())
        }
    }
}
