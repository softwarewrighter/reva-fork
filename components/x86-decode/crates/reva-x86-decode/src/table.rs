//! The opcode table and `decode`.

use crate::escape::{escape_0f, x87};
use crate::groups::{call_jmp, group3, group45, mov_moffs, singles};
use reva_x86_base::fault::Fault;
use reva_x86_cursor::cursor::{Cursor, Decoder, Prefixes};
use reva_x86_mem::memory::Memory;
use reva_x86_opcodes::alu::{alu_op, group1, mov_rm, test_rm, xchg_rm};
use reva_x86_opcodes::flow::{jcc8, loop_op, mov_rm_imm, ret, shift};
use reva_x86_opcodes::misc::{convert, mov_imm, string, test_eax, xchg_eax};
use reva_x86_opcodes::stack::{imul_imm, inc_dec_reg, lea_pop, push_imm, push_pop_reg};
use reva_x86_types::op::{Insn, Rep};
use reva_x86_types::operand::Size;

/// Opcode ranges and their decoders; a later range overrides an earlier.
const RANGES: &[(u8, u8, Decoder)] = &[
    (0x00, 0x3f, alu_op),
    (0x0f, 0x0f, escape_0f),
    (0x40, 0x4f, inc_dec_reg),
    (0x50, 0x5f, push_pop_reg),
    (0x68, 0x68, push_imm),
    (0x69, 0x69, imul_imm),
    (0x6a, 0x6a, push_imm),
    (0x6b, 0x6b, imul_imm),
    (0x70, 0x7f, jcc8),
    (0x80, 0x81, group1),
    (0x83, 0x83, group1),
    (0x84, 0x85, test_rm),
    (0x86, 0x87, xchg_rm),
    (0x88, 0x8b, mov_rm),
    (0x8d, 0x8d, lea_pop),
    (0x8f, 0x8f, lea_pop),
    (0x90, 0x90, singles),
    (0x91, 0x97, xchg_eax),
    (0x98, 0x99, convert),
    (0x9b, 0x9b, singles),
    (0xa0, 0xa3, mov_moffs),
    (0xa4, 0xa7, string),
    (0xa8, 0xa9, test_eax),
    (0xaa, 0xaf, string),
    (0xb0, 0xbf, mov_imm),
    (0xc0, 0xc1, shift),
    (0xc2, 0xc3, ret),
    (0xc6, 0xc7, mov_rm_imm),
    (0xcc, 0xcc, singles),
    (0xd0, 0xd3, shift),
    (0xd8, 0xdf, x87),
    (0xe0, 0xe3, loop_op),
    (0xe8, 0xe9, call_jmp),
    (0xeb, 0xeb, call_jmp),
    (0xf4, 0xf5, singles),
    (0xf6, 0xf7, group3),
    (0xf8, 0xf9, singles),
    (0xfc, 0xfd, singles),
    (0xfe, 0xff, group45),
];

/// The decoder of each opcode byte; `None`: not in the subset.
const TABLE: [Option<Decoder>; 256] = build(RANGES);

const fn build(ranges: &[(u8, u8, Decoder)]) -> [Option<Decoder>; 256] {
    let mut table: [Option<Decoder>; 256] = [None; 256];
    let mut i = 0;
    while i < ranges.len() {
        let (lo, hi, d) = ranges[i];
        let mut b = lo as usize;
        while b <= hi as usize {
            table[b] = Some(d);
            b += 1;
        }
        i += 1;
    }
    table
}

/// Decode the instruction at `eip`.
pub fn decode(mem: &Memory, eip: u32) -> Result<Insn, Fault> {
    let mut c = Cursor::new(mem, eip);
    let (p, b) = prefixes(&mut c)?;
    let op = match TABLE[b as usize] {
        Some(d) => d(&mut c, b, p)?,
        None => return Err(c.bad()),
    };
    let len = c.pos.wrapping_sub(eip);
    if len > 15 {
        return Err(c.bad());
    }
    Ok(Insn { op, len: len as u8 })
}

/// Up to four prefixes (`66`, `F3`, `F2`), and the byte after them.
fn prefixes(c: &mut Cursor) -> Result<(Prefixes, u8), Fault> {
    let mut p = Prefixes {
        osz: Size::D,
        rep: Rep::None,
    };
    let mut b = c.u8()?;
    for _ in 0..4 {
        match b {
            0x66 => p.osz = Size::W,
            0xf3 => p.rep = Rep::E,
            0xf2 => p.rep = Rep::Ne,
            _ => break,
        }
        b = c.u8()?;
    }
    Ok((p, b))
}
