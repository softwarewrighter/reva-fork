//! Decoder: bytes in VM memory → [`Insn`].
//!
//! Covers the one-byte and `0F`-escaped opcodes that appear in `newcore.asm`
//! output, `reva.f`, and the integer parts of the libraries, plus their
//! obvious neighbours (all eight ALU ops in every form, all shift kinds, all
//! condition codes).  The `66` operand-size prefix is honoured for the
//! generic `Ev`/`Gv`/`Iv` forms; `F2`/`F3` for string instructions.

use super::*;
use crate::fault::Fault;
use crate::mem::Memory;

const ALU: [Alu; 8] = [
    Alu::Add,
    Alu::Or,
    Alu::Adc,
    Alu::Sbb,
    Alu::And,
    Alu::Sub,
    Alu::Xor,
    Alu::Cmp,
];

const SHIFT: [Shift; 8] = [
    Shift::Rol,
    Shift::Ror,
    Shift::Rcl,
    Shift::Rcr,
    Shift::Shl,
    Shift::Shr,
    Shift::Shl, // /6 is an undocumented alias of SHL
    Shift::Sar,
];

struct Cursor<'a> {
    mem: &'a Memory,
    start: u32,
    pos: u32,
}

impl Cursor<'_> {
    fn u8(&mut self) -> Result<u8, Fault> {
        let b = self.mem.read_u8(self.pos)?;
        self.pos = self.pos.wrapping_add(1);
        Ok(b)
    }
    fn i8(&mut self) -> Result<u32, Fault> {
        Ok(self.u8()? as i8 as i32 as u32)
    }
    fn u16(&mut self) -> Result<u16, Fault> {
        let v = self.mem.read_u16(self.pos)?;
        self.pos = self.pos.wrapping_add(2);
        Ok(v)
    }
    fn u32(&mut self) -> Result<u32, Fault> {
        let v = self.mem.read_u32(self.pos)?;
        self.pos = self.pos.wrapping_add(4);
        Ok(v)
    }
    fn imm(&mut self, size: Size) -> Result<u32, Fault> {
        Ok(match size {
            Size::B => self.u8()? as u32,
            Size::W => self.u16()? as u32,
            Size::D => self.u32()?,
        })
    }
    /// Relative branch target: displacement is relative to the *end* of
    /// the instruction, which is the cursor position after reading it.
    fn rel8(&mut self) -> Result<u32, Fault> {
        let d = self.i8()?;
        Ok(self.pos.wrapping_add(d))
    }
    fn rel32(&mut self) -> Result<u32, Fault> {
        let d = self.u32()?;
        Ok(self.pos.wrapping_add(d))
    }

    /// ModRM (+SIB, +displacement).  Returns the `reg` field and the r/m
    /// operand.
    fn modrm(&mut self) -> Result<(u8, Opd), Fault> {
        let m = self.u8()?;
        let (md, reg, rm) = (m >> 6, (m >> 3) & 7, m & 7);
        if md == 3 {
            return Ok((reg, Opd::Reg(rm)));
        }
        let mut mr = MemRef {
            base: None,
            index: None,
            scale: 1,
            disp: 0,
        };
        if rm == 4 {
            let sib = self.u8()?;
            let (ss, idx, base) = (sib >> 6, (sib >> 3) & 7, sib & 7);
            if idx != 4 {
                mr.index = Some(idx);
                mr.scale = 1 << ss;
            }
            if base == 5 && md == 0 {
                mr.disp = self.u32()?;
            } else {
                mr.base = Some(base);
            }
        } else if rm == 5 && md == 0 {
            mr.disp = self.u32()?;
        } else {
            mr.base = Some(rm);
        }
        match md {
            1 => mr.disp = mr.disp.wrapping_add(self.i8()?),
            2 => mr.disp = mr.disp.wrapping_add(self.u32()?),
            _ => {}
        }
        Ok((reg, Opd::Mem(mr)))
    }

    fn bad(&self) -> Fault {
        let bytes = (0..8)
            .map_while(|i| self.mem.read_u8(self.start.wrapping_add(i)).ok())
            .collect();
        Fault::BadOpcode {
            eip: self.start,
            bytes,
        }
    }
}

/// Decode the instruction at `eip`.
pub fn decode(mem: &Memory, eip: u32) -> Result<Insn, Fault> {
    let mut c = Cursor {
        mem,
        start: eip,
        pos: eip,
    };
    let mut osz = Size::D;
    let mut rep = Rep::None;
    let mut b = c.u8()?;
    for _ in 0..4 {
        match b {
            0x66 => osz = Size::W,
            0xf3 => rep = Rep::E,
            0xf2 => rep = Rep::Ne,
            _ => break,
        }
        b = c.u8()?;
    }

    let op = match b {
        // ALU: 00-05 add, 08-0d or, 10-15 adc, 18-1d sbb, 20-25 and,
        //      28-2d sub, 30-35 xor, 38-3d cmp
        0x00..=0x3f if b & 7 < 6 => {
            let alu = ALU[(b >> 3) as usize];
            match b & 7 {
                0 => {
                    let (r, rm) = c.modrm()?;
                    Op::Alu {
                        op: alu,
                        size: Size::B,
                        dst: rm,
                        src: Opd::Reg(r),
                    }
                }
                1 => {
                    let (r, rm) = c.modrm()?;
                    Op::Alu {
                        op: alu,
                        size: osz,
                        dst: rm,
                        src: Opd::Reg(r),
                    }
                }
                2 => {
                    let (r, rm) = c.modrm()?;
                    Op::Alu {
                        op: alu,
                        size: Size::B,
                        dst: Opd::Reg(r),
                        src: rm,
                    }
                }
                3 => {
                    let (r, rm) = c.modrm()?;
                    Op::Alu {
                        op: alu,
                        size: osz,
                        dst: Opd::Reg(r),
                        src: rm,
                    }
                }
                4 => Op::Alu {
                    op: alu,
                    size: Size::B,
                    dst: Opd::Reg(EAX),
                    src: Opd::Imm(c.imm(Size::B)?),
                },
                _ => Op::Alu {
                    op: alu,
                    size: osz,
                    dst: Opd::Reg(EAX),
                    src: Opd::Imm(c.imm(osz)?),
                },
            }
        }
        0x40..=0x47 => Op::Inc {
            size: osz,
            dst: Opd::Reg(b - 0x40),
        },
        0x48..=0x4f => Op::Dec {
            size: osz,
            dst: Opd::Reg(b - 0x48),
        },
        0x50..=0x57 if osz == Size::D => Op::Push(Opd::Reg(b - 0x50)),
        0x58..=0x5f if osz == Size::D => Op::Pop(Opd::Reg(b - 0x58)),
        0x68 if osz == Size::D => Op::Push(Opd::Imm(c.u32()?)),
        0x6a if osz == Size::D => Op::Push(Opd::Imm(c.i8()?)),
        0x69 if osz == Size::D => {
            let (r, rm) = c.modrm()?;
            Op::Imul {
                dst: r,
                src: rm,
                imm: Some(c.u32()?),
            }
        }
        0x6b if osz == Size::D => {
            let (r, rm) = c.modrm()?;
            Op::Imul {
                dst: r,
                src: rm,
                imm: Some(c.i8()?),
            }
        }
        0x70..=0x7f => Op::Jcc {
            cc: b & 15,
            target: c.rel8()?,
        },
        0x80 | 0x81 | 0x83 => {
            let size = if b == 0x80 { Size::B } else { osz };
            let (r, rm) = c.modrm()?;
            let imm = match b {
                0x80 => c.imm(Size::B)?,
                0x81 => c.imm(osz)?,
                _ => c.i8()?,
            };
            Op::Alu {
                op: ALU[r as usize],
                size,
                dst: rm,
                src: Opd::Imm(imm),
            }
        }
        0x84 | 0x85 => {
            let size = if b == 0x84 { Size::B } else { osz };
            let (r, rm) = c.modrm()?;
            Op::Test {
                size,
                a: rm,
                b: Opd::Reg(r),
            }
        }
        0x86 | 0x87 => {
            let size = if b == 0x86 { Size::B } else { osz };
            let (r, rm) = c.modrm()?;
            Op::Xchg {
                size,
                a: rm,
                b: Opd::Reg(r),
            }
        }
        0x88..=0x8b => {
            let size = if b & 1 == 0 { Size::B } else { osz };
            let (r, rm) = c.modrm()?;
            if b & 2 == 0 {
                Op::Mov {
                    size,
                    dst: rm,
                    src: Opd::Reg(r),
                }
            } else {
                Op::Mov {
                    size,
                    dst: Opd::Reg(r),
                    src: rm,
                }
            }
        }
        0x8d => match c.modrm()? {
            (r, Opd::Mem(m)) => Op::Lea { dst: r, mem: m },
            _ => return Err(c.bad()),
        },
        0x8f => match c.modrm()? {
            (0, rm) => Op::Pop(rm),
            _ => return Err(c.bad()),
        },
        0x90 => Op::Nop,
        0x91..=0x97 => Op::Xchg {
            size: osz,
            a: Opd::Reg(EAX),
            b: Opd::Reg(b - 0x90),
        },
        0x98 if osz == Size::D => Op::Cwde,
        0x99 if osz == Size::D => Op::Cdq,
        0xa4..=0xa7 | 0xaa..=0xaf => {
            let size = if b & 1 == 0 { Size::B } else { osz };
            let op = match b & !1 {
                0xa4 => StrOp::Movs,
                0xa6 => StrOp::Cmps,
                0xaa => StrOp::Stos,
                0xac => StrOp::Lods,
                _ => StrOp::Scas,
            };
            Op::Str { op, size, rep }
        }
        0xa8 => Op::Test {
            size: Size::B,
            a: Opd::Reg(EAX),
            b: Opd::Imm(c.imm(Size::B)?),
        },
        0xa9 => Op::Test {
            size: osz,
            a: Opd::Reg(EAX),
            b: Opd::Imm(c.imm(osz)?),
        },
        0xb0..=0xb7 => Op::Mov {
            size: Size::B,
            dst: Opd::Reg(b - 0xb0),
            src: Opd::Imm(c.imm(Size::B)?),
        },
        0xb8..=0xbf => Op::Mov {
            size: osz,
            dst: Opd::Reg(b - 0xb8),
            src: Opd::Imm(c.imm(osz)?),
        },
        0xc0 | 0xc1 | 0xd0..=0xd3 => {
            let size = if b & 1 == 0 { Size::B } else { osz };
            let (r, rm) = c.modrm()?;
            let count = match b {
                0xc0 | 0xc1 => Opd::Imm(c.u8()? as u32),
                0xd0 | 0xd1 => Opd::Imm(1),
                _ => Opd::Reg(ECX), // CL
            };
            Op::Shift {
                op: SHIFT[r as usize],
                size,
                dst: rm,
                count,
            }
        }
        0xc2 => Op::Ret(c.u16()?),
        0xc3 => Op::Ret(0),
        0xc6 | 0xc7 => {
            let size = if b == 0xc6 { Size::B } else { osz };
            match c.modrm()? {
                (0, rm) => Op::Mov {
                    size,
                    dst: rm,
                    src: Opd::Imm(c.imm(size)?),
                },
                _ => return Err(c.bad()),
            }
        }
        0xcc => Op::Int3,
        0xe0..=0xe3 => {
            let kind = match b {
                0xe0 => LoopKind::Loopne,
                0xe1 => LoopKind::Loope,
                0xe2 => LoopKind::Loop,
                _ => LoopKind::Jecxz,
            };
            Op::Loop {
                kind,
                target: c.rel8()?,
            }
        }
        0xe8 => Op::Call(c.rel32()?),
        0xe9 => Op::Jmp(c.rel32()?),
        0xeb => Op::Jmp(c.rel8()?),
        0xf4 => Op::Hlt,
        0xf5 => Op::Flag(FlagOp::Cmc),
        0xf6 | 0xf7 => {
            let size = if b == 0xf6 { Size::B } else { osz };
            let (r, rm) = c.modrm()?;
            match r {
                0 | 1 => Op::Test {
                    size,
                    a: rm,
                    b: Opd::Imm(c.imm(size)?),
                },
                2 => Op::Not { size, dst: rm },
                3 => Op::Neg { size, dst: rm },
                4 => Op::MulDiv {
                    op: MulDiv::Mul,
                    size,
                    src: rm,
                },
                5 => Op::MulDiv {
                    op: MulDiv::Imul,
                    size,
                    src: rm,
                },
                6 => Op::MulDiv {
                    op: MulDiv::Div,
                    size,
                    src: rm,
                },
                _ => Op::MulDiv {
                    op: MulDiv::Idiv,
                    size,
                    src: rm,
                },
            }
        }
        0xf8 => Op::Flag(FlagOp::Clc),
        0xf9 => Op::Flag(FlagOp::Stc),
        0xfc => Op::Flag(FlagOp::Cld),
        0xfd => Op::Flag(FlagOp::Std),
        0xfe => match c.modrm()? {
            (0, rm) => Op::Inc {
                size: Size::B,
                dst: rm,
            },
            (1, rm) => Op::Dec {
                size: Size::B,
                dst: rm,
            },
            _ => return Err(c.bad()),
        },
        0xff => match c.modrm()? {
            (0, rm) => Op::Inc { size: osz, dst: rm },
            (1, rm) => Op::Dec { size: osz, dst: rm },
            (2, rm) => Op::CallInd(rm),
            (4, rm) => Op::JmpInd(rm),
            (6, rm) => Op::Push(rm),
            _ => return Err(c.bad()),
        },
        0x0f => {
            let b2 = c.u8()?;
            match b2 {
                0x80..=0x8f => Op::Jcc {
                    cc: b2 & 15,
                    target: c.rel32()?,
                },
                0x90..=0x9f => Op::Setcc {
                    cc: b2 & 15,
                    dst: c.modrm()?.1,
                },
                0x40..=0x4f if osz == Size::D => {
                    let (r, rm) = c.modrm()?;
                    Op::Cmovcc {
                        cc: b2 & 15,
                        dst: r,
                        src: rm,
                    }
                }
                0xaf if osz == Size::D => {
                    let (r, rm) = c.modrm()?;
                    Op::Imul {
                        dst: r,
                        src: rm,
                        imm: None,
                    }
                }
                0xb6 | 0xb7 | 0xbe | 0xbf => {
                    let (r, rm) = c.modrm()?;
                    let from = if b2 & 1 == 0 { Size::B } else { Size::W };
                    Op::Movx {
                        signed: b2 >= 0xbe,
                        from,
                        dst: r,
                        src: rm,
                    }
                }
                _ => return Err(c.bad()),
            }
        }
        _ => return Err(c.bad()),
    };
    let len = c.pos.wrapping_sub(eip);
    if len > 15 {
        return Err(c.bad());
    }
    Ok(Insn { op, len: len as u8 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(bytes: &[u8]) -> Insn {
        let mut m = Memory::new(0x3000);
        m.write_bytes(0x2000, bytes).unwrap();
        decode(&m, 0x2000).unwrap()
    }

    fn esi(disp: u32) -> Opd {
        Opd::Mem(MemRef {
            base: Some(ESI),
            index: None,
            scale: 1,
            disp,
        })
    }

    #[test]
    fn reva_dup() {
        // lea esi,[esi-4] ; mov [esi],eax
        let i = dec(&[0x8d, 0x76, 0xfc]);
        assert_eq!(i.len, 3);
        assert_eq!(
            i.op,
            Op::Lea {
                dst: ESI,
                mem: MemRef {
                    base: Some(ESI),
                    index: None,
                    scale: 1,
                    disp: 0xffff_fffc
                }
            }
        );
        let i = dec(&[0x89, 0x06]);
        assert_eq!(
            i.op,
            Op::Mov {
                size: Size::D,
                dst: esi(0),
                src: Opd::Reg(EAX)
            }
        );
    }

    #[test]
    fn reva_plus_and_lodsd() {
        let i = dec(&[0x01, 0x06]);
        assert_eq!(
            i.op,
            Op::Alu {
                op: Alu::Add,
                size: Size::D,
                dst: esi(0),
                src: Opd::Reg(EAX)
            }
        );
        let i = dec(&[0xad]);
        assert_eq!(
            i.op,
            Op::Str {
                op: StrOp::Lods,
                size: Size::D,
                rep: Rep::None
            }
        );
        assert_eq!(i.len, 1);
    }

    #[test]
    fn branches_are_absolute() {
        let i = dec(&[0xe8, 0x10, 0, 0, 0]);
        assert_eq!(i.op, Op::Call(0x2015));
        let i = dec(&[0x75, 0x01]);
        assert_eq!(
            i.op,
            Op::Jcc {
                cc: 5,
                target: 0x2003
            }
        );
        let i = dec(&[0x0f, 0x84, 0xfa, 0xff, 0xff, 0xff]);
        assert_eq!(
            i.op,
            Op::Jcc {
                cc: 4,
                target: 0x2000
            }
        );
        let i = dec(&[0xff, 0x15, 0x00, 0x21, 0, 0]);
        assert_eq!(
            i.op,
            Op::CallInd(Opd::Mem(MemRef {
                base: None,
                index: None,
                scale: 1,
                disp: 0x2100
            }))
        );
    }

    #[test]
    fn sib_and_prefixes() {
        // mov eax,[esp+8]
        let i = dec(&[0x8b, 0x44, 0x24, 0x08]);
        assert_eq!(
            i.op,
            Op::Mov {
                size: Size::D,
                dst: Opd::Reg(EAX),
                src: Opd::Mem(MemRef {
                    base: Some(ESP),
                    index: None,
                    scale: 1,
                    disp: 8
                })
            }
        );
        let i = dec(&[0xf3, 0xa4]);
        assert_eq!(
            i.op,
            Op::Str {
                op: StrOp::Movs,
                size: Size::B,
                rep: Rep::E
            }
        );
        let i = dec(&[0x66, 0xb8, 0x34, 0x12]);
        assert_eq!(
            i.op,
            Op::Mov {
                size: Size::W,
                dst: Opd::Reg(EAX),
                src: Opd::Imm(0x1234)
            }
        );
    }

    #[test]
    fn unsupported_reports_bytes() {
        let mut m = Memory::new(0x3000);
        m.write_bytes(0x2000, &[0xd9, 0xe8, 0x90]).unwrap(); // fld1
        match decode(&m, 0x2000) {
            Err(Fault::BadOpcode { eip, bytes }) => {
                assert_eq!(eip, 0x2000);
                assert_eq!(&bytes[..3], &[0xd9, 0xe8, 0x90]);
            }
            other => panic!("{other:?}"),
        }
    }
}
