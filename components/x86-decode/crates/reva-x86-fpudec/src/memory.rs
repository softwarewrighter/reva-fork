//! `DB`, `DD`, `DF` with memory (integer, 64-bit and 80-bit forms), and
//! `DB`/`DD` with a register.

use crate::load::store;
use reva_x86_types::fpu_op::{FMem, FSrc, FpuOp};
use reva_x86_types::operand::MemRef;

/// `DB` with memory: fild m32, fisttp/fist/fistp m32, fld/fstp m80.
pub fn db_mem(r: u8, m: MemRef) -> Option<FpuOp> {
    Some(match r {
        0 => FpuOp::Load(FSrc::Mem(FMem::I32, m)),
        1..=3 => store(FMem::I32, m, r != 2, r == 1),
        5 => FpuOp::Load(FSrc::Mem(FMem::F80, m)),
        7 => store(FMem::F80, m, true, false),
        _ => return None,
    })
}

/// `DB` with a register: fcmovnb, fcmovne, fcmovnbe, fcmovnu, fnclex,
/// fninit, fucomi, fcomi.
pub fn db_reg(r: u8, i: u8) -> Option<FpuOp> {
    Some(match (r, i) {
        (0..=3, _) => FpuOp::Cmov {
            cc: [0x3, 0x5, 0x7, 0xb][r as usize],
            i,
        },
        (4, 2) => FpuOp::Clex,
        (4, 3) => FpuOp::Init,
        (5 | 6, _) => FpuOp::Comi {
            i,
            pop: false,
            unordered: r == 5,
        },
        _ => return None,
    })
}

/// `DD` with memory: fld m64, fisttp m64, fst/fstp m64, fnstsw m16.
pub fn dd_mem(r: u8, m: MemRef) -> Option<FpuOp> {
    Some(match r {
        0 => FpuOp::Load(FSrc::Mem(FMem::F64, m)),
        1 => store(FMem::I64, m, true, true),
        2 | 3 => store(FMem::F64, m, r == 3, false),
        7 => FpuOp::Stsw(Some(m)),
        _ => return None,
    })
}

/// `DD` with a register: ffree, fst/fstp st(i), fucom/fucomp.
pub fn dd_reg(r: u8, i: u8) -> Option<FpuOp> {
    Some(match r {
        0 => FpuOp::Free(i),
        2 | 3 => FpuOp::StoreSt { i, pop: r == 3 },
        4 | 5 => FpuOp::Compare {
            src: FSrc::St(i),
            pops: u8::from(r == 5),
            unordered: true,
        },
        _ => return None,
    })
}

/// `DF` with memory: fild m16, fisttp/fist/fistp m16, fild/fistp m64.
pub fn df_mem(r: u8, m: MemRef) -> Option<FpuOp> {
    Some(match r {
        0 => FpuOp::Load(FSrc::Mem(FMem::I16, m)),
        1..=3 => store(FMem::I16, m, r != 2, r == 1),
        5 => FpuOp::Load(FSrc::Mem(FMem::I64, m)),
        7 => store(FMem::I64, m, true, false),
        _ => return None,
    })
}
