//! Executor: apply one [`Insn`] to [`Cpu`] + [`Memory`].
//!
//! Flags follow the Intel SDM for every instruction in the subset, except AF
//! (not modelled) and the "undefined" flag results, which are left as the
//! nearest sensible value.  Reva code tests ZF, CF, SF and OF (`jz`, `jc`,
//! `sbb eax,eax`, `jle`) and occasionally PF.

use super::*;
use crate::fault::Fault;
use crate::mem::Memory;

impl Cpu {
    #[inline]
    pub fn ea(&self, m: &MemRef) -> u32 {
        let mut a = m.disp;
        if let Some(b) = m.base {
            a = a.wrapping_add(self.r[b as usize]);
        }
        if let Some(i) = m.index {
            a = a.wrapping_add(self.r[i as usize].wrapping_mul(m.scale as u32));
        }
        a
    }

    #[inline]
    pub fn get_reg(&self, n: u8, size: Size) -> u32 {
        let n = n as usize;
        match size {
            Size::D => self.r[n],
            Size::W => self.r[n] & 0xffff,
            Size::B if n < 4 => self.r[n] & 0xff,
            Size::B => (self.r[n - 4] >> 8) & 0xff,
        }
    }

    #[inline]
    pub fn set_reg(&mut self, n: u8, size: Size, v: u32) {
        let n = n as usize;
        match size {
            Size::D => self.r[n] = v,
            Size::W => self.r[n] = (self.r[n] & 0xffff_0000) | (v & 0xffff),
            Size::B if n < 4 => self.r[n] = (self.r[n] & !0xff) | (v & 0xff),
            Size::B => self.r[n - 4] = (self.r[n - 4] & !0xff00) | ((v & 0xff) << 8),
        }
    }

    #[inline]
    fn read(&self, mem: &Memory, o: Opd, size: Size) -> Result<u32, Fault> {
        match o {
            Opd::Reg(n) => Ok(self.get_reg(n, size)),
            Opd::Imm(v) => Ok(v & size.mask()),
            Opd::Mem(m) => {
                let a = self.ea(&m);
                match size {
                    Size::B => mem.read_u8(a).map(u32::from),
                    Size::W => mem.read_u16(a).map(u32::from),
                    Size::D => mem.read_u32(a),
                }
            }
        }
    }

    #[inline]
    fn write(&mut self, mem: &mut Memory, o: Opd, size: Size, v: u32) -> Result<(), Fault> {
        match o {
            Opd::Reg(n) => {
                self.set_reg(n, size, v);
                Ok(())
            }
            Opd::Mem(m) => {
                let a = self.ea(&m);
                match size {
                    Size::B => mem.write_u8(a, v as u8),
                    Size::W => mem.write_u16(a, v as u16),
                    Size::D => mem.write_u32(a, v),
                }
            }
            Opd::Imm(_) => Err(Fault::Trap {
                eip: self.eip,
                what: "write to immediate",
            }),
        }
    }

    #[inline]
    fn set_szp(&mut self, r: u32, size: Size) {
        self.zf = r & size.mask() == 0;
        self.sf = r & size.sign() != 0;
        self.pf = (r as u8).count_ones().is_multiple_of(2);
    }

    /// The eight ALU operations with full flag semantics.
    pub fn alu(&mut self, op: Alu, size: Size, a: u32, b: u32) -> u32 {
        let (m, s) = (size.mask(), size.sign());
        let (a, b) = (a & m, b & m);
        let r = match op {
            Alu::Add | Alu::Adc => {
                let c = (op == Alu::Adc && self.cf) as u64;
                let full = a as u64 + b as u64 + c;
                let r = full as u32 & m;
                self.cf = full > m as u64;
                self.of = (a ^ r) & (b ^ r) & s != 0;
                r
            }
            Alu::Sub | Alu::Sbb | Alu::Cmp => {
                let c = (op == Alu::Sbb && self.cf) as u32;
                let r = a.wrapping_sub(b).wrapping_sub(c) & m;
                self.cf = (a as u64) < b as u64 + c as u64;
                self.of = (a ^ b) & (a ^ r) & s != 0;
                r
            }
            Alu::And | Alu::Or | Alu::Xor => {
                self.cf = false;
                self.of = false;
                match op {
                    Alu::And => a & b,
                    Alu::Or => a | b,
                    _ => a ^ b,
                }
            }
        };
        self.set_szp(r, size);
        r
    }

    #[inline]
    pub fn push32(&mut self, mem: &mut Memory, v: u32) -> Result<(), Fault> {
        let sp = self.r[ESP as usize].wrapping_sub(4);
        if sp < self.esp_floor {
            return Err(Fault::ReturnStackOverflow);
        }
        mem.write_u32(sp, v)?;
        self.r[ESP as usize] = sp;
        Ok(())
    }

    #[inline]
    pub fn pop32(&mut self, mem: &Memory) -> Result<u32, Fault> {
        let sp = self.r[ESP as usize];
        let v = mem.read_u32(sp)?;
        self.r[ESP as usize] = sp.wrapping_add(4);
        Ok(v)
    }

    fn shift(&mut self, op: Shift, size: Size, v: u32, cnt: u32) -> u32 {
        let bits = size.bytes() * 8;
        let (m, s) = (size.mask(), size.sign());
        let v = v & m;
        match op {
            Shift::Shl => {
                let full = (v as u64) << cnt;
                let r = full as u32 & m;
                self.cf = (full >> bits) & 1 != 0;
                self.of = (r & s != 0) != self.cf;
                self.set_szp(r, size);
                r
            }
            Shift::Shr => {
                let r = ((v as u64) >> cnt) as u32;
                self.cf = ((v as u64) >> (cnt - 1)) & 1 != 0;
                self.of = v & s != 0;
                self.set_szp(r, size);
                r
            }
            Shift::Sar => {
                let sv = if v & s != 0 {
                    (v as i64) - ((m as i64) + 1)
                } else {
                    v as i64
                };
                let r = (sv >> cnt.min(63)) as u32 & m;
                self.cf = (sv >> (cnt - 1).min(63)) & 1 != 0;
                self.of = false;
                self.set_szp(r, size);
                r
            }
            Shift::Rol | Shift::Ror => {
                let c = cnt % bits;
                let r = if c == 0 {
                    v
                } else if op == Shift::Rol {
                    ((v << c) | (v >> (bits - c))) & m
                } else {
                    ((v >> c) | (v << (bits - c))) & m
                };
                if op == Shift::Rol {
                    self.cf = r & 1 != 0;
                    self.of = (r & s != 0) != self.cf;
                } else {
                    self.cf = r & s != 0;
                    self.of = (r & s != 0) != (r & (s >> 1) != 0);
                }
                r
            }
            Shift::Rcl | Shift::Rcr => {
                let mut r = v;
                for _ in 0..cnt % (bits + 1) {
                    if op == Shift::Rcl {
                        let out = r & s != 0;
                        r = ((r << 1) | self.cf as u32) & m;
                        self.cf = out;
                    } else {
                        let out = r & 1 != 0;
                        r = (r >> 1) | if self.cf { s } else { 0 };
                        self.cf = out;
                    }
                }
                self.of = if op == Shift::Rcl {
                    (r & s != 0) != self.cf
                } else {
                    (r & s != 0) != (r & (s >> 1) != 0)
                };
                r
            }
        }
    }

    fn muldiv(&mut self, op: MulDiv, size: Size, src: u32, at: u32) -> Result<(), Fault> {
        let err = Fault::DivideError { eip: at };
        let bits = size.bytes() * 8;
        let m = size.mask() as u64;
        // Accumulator pair: AX for bytes (AL/AH), DX:AX, EDX:EAX.
        let (lo, hi) = match size {
            Size::B => (self.r[0] & 0xff, (self.r[0] >> 8) & 0xff),
            Size::W => (self.r[0] & 0xffff, self.r[2] & 0xffff),
            Size::D => (self.r[0], self.r[2]),
        };
        let sext = |v: u64, b: u32| -> i64 { ((v << (64 - b)) as i64) >> (64 - b) };
        let (q, r) = match op {
            MulDiv::Mul => {
                let p = lo as u64 * src as u64;
                let (q, r) = (p & m, (p >> bits) & m);
                self.cf = r != 0;
                self.of = r != 0;
                (q, r)
            }
            MulDiv::Imul => {
                let p = sext(lo as u64, bits) * sext(src as u64, bits);
                let fits = sext(p as u64 & m, bits) == p;
                self.cf = !fits;
                self.of = !fits;
                (p as u64 & m, (p as u64 >> bits) & m)
            }
            MulDiv::Div => {
                let n = ((hi as u64) << bits) | lo as u64;
                if src == 0 {
                    return Err(err);
                }
                let q = n / src as u64;
                if q > m {
                    return Err(err);
                }
                (q, n % src as u64)
            }
            MulDiv::Idiv => {
                let n = sext(((hi as u64) << bits) | lo as u64, bits * 2);
                let d = sext(src as u64, bits);
                let q = n.checked_div(d).ok_or(err.clone())?;
                if sext(q as u64 & m, bits) != q {
                    return Err(err);
                }
                (q as u64 & m, (n % d) as u64 & m)
            }
        };
        match size {
            Size::B => self.set_reg(EAX, Size::W, (q as u32) | ((r as u32) << 8)),
            Size::W => {
                self.set_reg(EAX, Size::W, q as u32);
                self.set_reg(EDX, Size::W, r as u32);
            }
            Size::D => {
                self.r[0] = q as u32;
                self.r[2] = r as u32;
            }
        }
        Ok(())
    }

    fn string_once(&mut self, mem: &mut Memory, op: StrOp, size: Size) -> Result<(), Fault> {
        let n = size.bytes();
        let step = if self.df { n.wrapping_neg() } else { n };
        let (si, di) = (ESI as usize, EDI as usize);
        let rd = |mem: &Memory, a: u32| -> Result<u32, Fault> {
            match size {
                Size::B => mem.read_u8(a).map(u32::from),
                Size::W => mem.read_u16(a).map(u32::from),
                Size::D => mem.read_u32(a),
            }
        };
        let wr = |mem: &mut Memory, a: u32, v: u32| -> Result<(), Fault> {
            match size {
                Size::B => mem.write_u8(a, v as u8),
                Size::W => mem.write_u16(a, v as u16),
                Size::D => mem.write_u32(a, v),
            }
        };
        match op {
            StrOp::Lods => {
                let v = rd(mem, self.r[si])?;
                self.set_reg(EAX, size, v);
                self.r[si] = self.r[si].wrapping_add(step);
            }
            StrOp::Stos => {
                wr(mem, self.r[di], self.get_reg(EAX, size))?;
                self.r[di] = self.r[di].wrapping_add(step);
            }
            StrOp::Movs => {
                let v = rd(mem, self.r[si])?;
                wr(mem, self.r[di], v)?;
                self.r[si] = self.r[si].wrapping_add(step);
                self.r[di] = self.r[di].wrapping_add(step);
            }
            StrOp::Scas => {
                let b = rd(mem, self.r[di])?;
                self.alu(Alu::Cmp, size, self.get_reg(EAX, size), b);
                self.r[di] = self.r[di].wrapping_add(step);
            }
            StrOp::Cmps => {
                let a = rd(mem, self.r[si])?;
                let b = rd(mem, self.r[di])?;
                self.alu(Alu::Cmp, size, a, b);
                self.r[si] = self.r[si].wrapping_add(step);
                self.r[di] = self.r[di].wrapping_add(step);
            }
        }
        Ok(())
    }

    /// Execute `insn`, which was decoded at `self.eip`.
    pub fn exec(&mut self, mem: &mut Memory, insn: &Insn) -> Result<(), Fault> {
        let at = self.eip;
        let next = at.wrapping_add(insn.len as u32);
        self.eip = next;
        match insn.op {
            Op::Alu { op, size, dst, src } => {
                let a = self.read(mem, dst, size)?;
                let b = self.read(mem, src, size)?;
                let r = self.alu(op, size, a, b);
                if op != Alu::Cmp {
                    self.write(mem, dst, size, r)?;
                }
            }
            Op::Test { size, a, b } => {
                let x = self.read(mem, a, size)?;
                let y = self.read(mem, b, size)?;
                self.alu(Alu::And, size, x, y);
            }
            Op::Mov { size, dst, src } => {
                let v = self.read(mem, src, size)?;
                self.write(mem, dst, size, v)?;
            }
            Op::Movx {
                signed,
                from,
                dst,
                src,
            } => {
                let v = self.read(mem, src, from)?;
                self.r[dst as usize] = if signed && v & from.sign() != 0 {
                    v | !from.mask()
                } else {
                    v
                };
            }
            Op::Lea { dst, mem: m } => self.r[dst as usize] = self.ea(&m),
            Op::Xchg { size, a, b } => {
                let x = self.read(mem, a, size)?;
                let y = self.read(mem, b, size)?;
                self.write(mem, a, size, y)?;
                self.write(mem, b, size, x)?;
            }
            Op::Inc { size, dst } | Op::Dec { size, dst } => {
                let v = self.read(mem, dst, size)?;
                let c = self.cf;
                let op = if matches!(insn.op, Op::Inc { .. }) {
                    Alu::Add
                } else {
                    Alu::Sub
                };
                let r = self.alu(op, size, v, 1);
                self.cf = c;
                self.write(mem, dst, size, r)?;
            }
            Op::Not { size, dst } => {
                let v = self.read(mem, dst, size)?;
                self.write(mem, dst, size, !v)?;
            }
            Op::Neg { size, dst } => {
                let v = self.read(mem, dst, size)?;
                let r = self.alu(Alu::Sub, size, 0, v);
                self.write(mem, dst, size, r)?;
            }
            Op::Push(o) => {
                let v = self.read(mem, o, Size::D)?;
                self.push32(mem, v)?;
            }
            Op::Pop(o) => {
                let v = self.pop32(mem)?;
                self.write(mem, o, Size::D, v)?;
            }
            Op::Shift {
                op,
                size,
                dst,
                count,
            } => {
                let cnt = self.read(mem, count, Size::B)? & 0x1f;
                if cnt != 0 {
                    let v = self.read(mem, dst, size)?;
                    let r = self.shift(op, size, v, cnt);
                    self.write(mem, dst, size, r)?;
                }
            }
            Op::MulDiv { op, size, src } => {
                let s = self.read(mem, src, size)?;
                self.muldiv(op, size, s, at)?;
            }
            Op::Imul { dst, src, imm } => {
                let s = self.read(mem, src, Size::D)? as i32 as i64;
                let o = match imm {
                    Some(i) => i as i32 as i64,
                    None => self.r[dst as usize] as i32 as i64,
                };
                let p = s * o;
                let fits = p == p as i32 as i64;
                self.cf = !fits;
                self.of = !fits;
                self.r[dst as usize] = p as u32;
            }
            Op::Cdq => {
                self.r[2] = if self.r[0] & 0x8000_0000 != 0 {
                    u32::MAX
                } else {
                    0
                }
            }
            Op::Cwde => self.r[0] = self.r[0] as u16 as i16 as i32 as u32,
            Op::Str { op, size, rep } => match rep {
                Rep::None => self.string_once(mem, op, size)?,
                _ => {
                    let cmp = matches!(op, StrOp::Scas | StrOp::Cmps);
                    while self.r[ECX as usize] != 0 {
                        self.string_once(mem, op, size)?;
                        self.r[ECX as usize] -= 1;
                        if cmp && ((rep == Rep::E && !self.zf) || (rep == Rep::Ne && self.zf)) {
                            break;
                        }
                    }
                }
            },
            Op::Jcc { cc, target } => {
                if self.cond(cc) {
                    self.eip = target;
                }
            }
            Op::Setcc { cc, dst } => {
                let v = self.cond(cc) as u32;
                self.write(mem, dst, Size::B, v)?;
            }
            Op::Bswap(r) => self.r[r as usize] = self.r[r as usize].swap_bytes(),
            Op::Cmovcc { cc, dst, src } => {
                let v = self.read(mem, src, Size::D)?;
                if self.cond(cc) {
                    self.r[dst as usize] = v;
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
            Op::Loop { kind, target } => {
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
            Op::Flag(f) => match f {
                FlagOp::Clc => self.cf = false,
                FlagOp::Stc => self.cf = true,
                FlagOp::Cmc => self.cf = !self.cf,
                FlagOp::Cld => self.df = false,
                FlagOp::Std => self.df = true,
            },
            Op::Nop => {}
            Op::Hlt => {
                return Err(Fault::Trap {
                    eip: at,
                    what: "hlt",
                });
            }
            Op::Int3 => {
                return Err(Fault::Trap {
                    eip: at,
                    what: "int3 breakpoint",
                });
            }
            Op::Fpu(f) => self.fpu_exec(mem, f)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x86::decode;

    /// Run `code` placed at 0x2000 with the given data stack, Reva-style:
    /// `stack` is bottom..top, TOS goes in EAX, the rest below ESI.
    fn run(code: &[u8], stack: &[u32]) -> (Cpu, Vec<u32>) {
        let mut m = Memory::new(0x10000);
        m.write_bytes(0x2000, code).unwrap();
        m.write_u8(0x2000 + code.len() as u32, 0xf4).unwrap(); // hlt sentinel
        let mut c = Cpu {
            esp_floor: 0x8000,
            ..Cpu::default()
        };
        c.r[ESP as usize] = 0x9000;
        let s0 = 0x6000u32;
        let mut sp = s0;
        let (tos, rest) = stack.split_last().map(|(t, r)| (*t, r)).unwrap_or((0, &[]));
        for v in rest {
            sp -= 4;
            m.write_u32(sp, *v).unwrap();
        }
        c.r[ESI as usize] = sp;
        c.r[EAX as usize] = tos;
        c.eip = 0x2000;
        loop {
            let i = decode(&m, c.eip).unwrap();
            if i.op == Op::Hlt {
                break;
            }
            c.exec(&mut m, &i).unwrap();
            if c.eip >= 0x2000 + code.len() as u32 || c.eip < 0x2000 {
                break;
            }
        }
        let mut out = Vec::new();
        let mut a = s0;
        while a > c.r[ESI as usize] {
            a -= 4;
            out.push(m.read_u32(a).unwrap());
        }
        out.push(c.r[EAX as usize]);
        (c, out)
    }

    // The byte sequences below are copied verbatim from src/reva.f.

    #[test]
    fn plus_minus_from_reva_f() {
        // : + inline{ 01 06 } drop ;   drop = 8b 06 8d 76 04
        assert_eq!(
            run(&[0x01, 0x06, 0x8b, 0x06, 0x8d, 0x76, 0x04], &[2, 3]).1,
            vec![5]
        );
        // : - inline{ 29 06 } drop ;
        assert_eq!(
            run(&[0x29, 0x06, 0x8b, 0x06, 0x8d, 0x76, 0x04], &[10, 3]).1,
            vec![7]
        );
    }

    #[test]
    fn swap_rot_star() {
        // : swap inline{ 89 c3 8b 06 89 1e }
        assert_eq!(
            run(&[0x89, 0xc3, 0x8b, 0x06, 0x89, 0x1e], &[1, 2]).1,
            vec![2, 1]
        );
        // : rot inline{ 50 8B 46 04 8B 1E 89 5E 04 8F 06 }
        let rot = [
            0x50, 0x8b, 0x46, 0x04, 0x8b, 0x1e, 0x89, 0x5e, 0x04, 0x8f, 0x06,
        ];
        assert_eq!(run(&rot, &[1, 2, 3]).1, vec![2, 3, 1]);
        // : * inline{ f7 26 83 c6 04 }   (mul dword [esi]; add esi,4)
        assert_eq!(run(&[0xf7, 0x26, 0x83, 0xc6, 0x04], &[6, 7]).1, vec![42]);
    }

    #[test]
    fn comparisons_via_flags() {
        // ((cmp)) = 3b 06 ad b8 00 00 00 00 ; < adds 7e 01 48
        let lt = [0x3b, 0x06, 0xad, 0xb8, 0, 0, 0, 0, 0x7e, 0x01, 0x48];
        assert_eq!(run(&lt, &[1, 2]).1, vec![u32::MAX]);
        assert_eq!(run(&lt, &[2, 1]).1, vec![0]);
        assert_eq!(run(&lt, &[(-5i32) as u32, 3]).1, vec![u32::MAX]);
        // : not inline{ 83 f8 01 19 c0 }   (cmp eax,1 ; sbb eax,eax)
        assert_eq!(run(&[0x83, 0xf8, 0x01, 0x19, 0xc0], &[0]).1, vec![u32::MAX]);
        assert_eq!(run(&[0x83, 0xf8, 0x01, 0x19, 0xc0], &[9]).1, vec![0]);
    }

    #[test]
    fn division_and_errors() {
        // : / inline{ 89 C3 AD 99 F7 FB }
        let div = [0x89, 0xc3, 0xad, 0x99, 0xf7, 0xfb];
        assert_eq!(run(&div, &[(-7i32) as u32, 2]).1, vec![(-3i32) as u32]);
        let mut m = Memory::new(0x10000);
        m.write_bytes(0x2000, &div).unwrap();
        let mut c = Cpu {
            eip: 0x2000,
            ..Cpu::default()
        };
        c.r[ESI as usize] = 0x6000;
        m.write_u32(0x6000, 1).unwrap();
        for _ in 0..3 {
            let i = decode(&m, c.eip).unwrap();
            c.exec(&mut m, &i).unwrap();
        }
        let i = decode(&m, c.eip).unwrap();
        assert_eq!(c.exec(&mut m, &i), Err(Fault::DivideError { eip: 0x2004 }));
    }

    #[test]
    fn string_ops_and_shifts() {
        // between: 89 c3 ad 89 c1 ad 93 40 29 c8 29 cb 29 c3 19 c0
        let between = [
            0x89, 0xc3, 0xad, 0x89, 0xc1, 0xad, 0x93, 0x40, 0x29, 0xc8, 0x29, 0xcb, 0x29, 0xc3,
            0x19, 0xc0,
        ];
        assert_eq!(run(&between, &[5, 1, 9]).1, vec![u32::MAX]);
        assert_eq!(run(&between, &[10, 1, 9]).1, vec![0]);
        // : << inline{ 89 c1 ad d3 e0 }
        assert_eq!(run(&[0x89, 0xc1, 0xad, 0xd3, 0xe0], &[3, 4]).1, vec![48]);
        // : >> inline{ 89 c1 ad d3 e8 }
        assert_eq!(
            run(&[0x89, 0xc1, 0xad, 0xd3, 0xe8], &[0x8000_0000, 31]).1,
            vec![1]
        );
    }

    #[test]
    fn eight_bit_registers() {
        let mut c = Cpu::default();
        c.r[EBX as usize] = 0x1122_3344;
        assert_eq!(c.get_reg(7, Size::B), 0x33); // BH
        c.set_reg(3, Size::B, 0xaa); // BL
        c.set_reg(7, Size::B, 0xbb); // BH
        assert_eq!(c.r[EBX as usize], 0x1122_bbaa);
    }
}
