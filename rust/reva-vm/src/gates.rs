//! # Gateways: the `newcore.asm` routines, in Rust
//!
//! Each [`Gate`] occupies one 4-byte slot in the gateway region.  Forth code
//! reaches it exactly as it reached the assembly routine (`call`, `jmp`,
//! `call [vector]`), and the routine sees the same registers: EAX = TOS,
//! ESI = data stack, ESP = return stack, EDX = dictionary entry for class
//! handlers.
//!
//! Comments name the original routine (`PROC xyz` in `src/newcore.asm`) and
//! call out every place where behaviour deliberately differs.
//!
//! Routines that must call back into Forth (the outer interpreter, `eval`,
//! `include`, `catch`, `:` via the `(header)` vector) are split into a head
//! and continuation gateways, so the VM never holds Forth state on the Rust
//! stack and can pause for input anywhere (see [`crate::machine`]).

use crate::fault::Fault;
use crate::host::Host;
use crate::layout::*;
use crate::machine::{st, Machine, Next};
use crate::x86::{EAX, EBX, EDI, EDX, ESI, ESP};

macro_rules! gates {
    ($($v:ident),* $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u16)]
        pub enum Gate { $($v),* }
        impl Gate {
            pub const ALL: &'static [Gate] = &[$(Gate::$v),*];
            pub fn name(self) -> &'static str {
                match self { $(Gate::$v => stringify!($v)),* }
            }
        }
    };
}

gates! {
    // word classes (the CLASS field of a dictionary entry points at one)
    FClass, FClassNotail, MClass, MClassNotail, IClass, CClass, VClass,
    ValClass, DoesClass, DClass,
    // default targets of DEFER vectors
    Noop, OsEmit, OsType, OsKey, OsBye, Find, WordNotFound, HeaderDefault, Hello,
    // startup and the outer interpreter, split at every call into Forth
    Cold, ColdDone, Repl, Interpret, QueryKey, QueryKeyGot, InterpWord,
    InterpFound, InterpIsWord, Eval, EvalDone, Include, Include2, IncludeDone,
    Catch, CatchDone, Throw, NestedReturn, OnStartThunk, Tick, TickDone, Colon,
    Variable, Variable2, Variable2Done,
    // the remaining core words
    MakeExe, Syscall, Reset, LzPack, LzDepack, LzMax, OnExit, OnStartup,
    LoadLib, UnloadLib, CallFn, OsFunc, FnvHash, Bye, Cmp, Cmpi, Place,
    PlusPlace, Move, Fill, Zt, Zcount, OpenR, OpenRw, Creat, Close, Read, Write,
    Fsize, Allocate, Free, Resize, Header, Literal, SSemi, InlineSemi, Semi,
    LBracket, RBracket, Parsews, Align, StoreHere, Compile, Single, Double,
    FromDigit, ToDigit, TwoDup, Comma, Comma3, Comma2, Comma1, Parse, ParseEsc,
    Save, PrintR, Link, Link2, Iterate, Slurp, SetExtra, GetExtra,
    // functions of the virtual libc, reached through `(call)` (see ffi.rs)
    HostGetenv, HostPutenv, HostLocaltime,
    // end of the embedded boot: restart as the saved executable would
    BootRestart,
}

impl Gate {
    #[inline]
    pub fn addr(self) -> u32 {
        GATE_BASE + GATE_STRIDE * self as u32
    }

    #[inline]
    pub fn from_addr(a: u32) -> Option<Gate> {
        let off = a.checked_sub(GATE_BASE)?;
        if off % GATE_STRIDE != 0 {
            return None;
        }
        Gate::ALL.get((off / GATE_STRIDE) as usize).copied()
    }
}

const SI: usize = ESI as usize;
const BASES: [u32; 5] = [10, 16, 2, 8, 255]; // prefixes # $ % & '
const DIGITS: &[u8; 16] = b"0123456789ABCDEF";

fn digit_value(c: u32, base: u32) -> Option<u32> {
    let v = match c {
        0x30..=0x39 => c - 0x30,
        0x41..=0x5a => c - 0x41 + 10,
        0x61..=0x7a => c - 0x61 + 10,
        _ => return None,
    };
    (v < base).then_some(v)
}

/// Result of a gateway helper.
type R<T = ()> = Result<T, Fault>;

impl Machine {
    // ---- register / stack helpers (the upsh/upop/drop macros) ---------

    #[inline]
    fn eax(&self) -> u32 {
        self.cpu.r[EAX as usize]
    }
    #[inline]
    fn set_eax(&mut self, v: u32) {
        self.cpu.r[EAX as usize] = v;
    }
    #[inline]
    fn nos(&self) -> R<u32> {
        self.mem.read_u32(self.cpu.r[SI])
    }
    #[inline]
    fn set_nos(&mut self, v: u32) -> R {
        self.mem.write_u32(self.cpu.r[SI], v)
    }
    #[inline]
    fn dup(&mut self) -> R {
        let sp = self.cpu.r[SI].wrapping_sub(4);
        self.mem.write_u32(sp, self.eax())?;
        self.cpu.r[SI] = sp;
        Ok(())
    }
    #[inline]
    fn upsh(&mut self, v: u32) -> R {
        self.dup()?;
        self.set_eax(v);
        Ok(())
    }
    #[inline]
    fn drop1(&mut self) -> R {
        let sp = self.cpu.r[SI];
        let v = self.mem.read_u32(sp)?;
        self.set_eax(v);
        self.cpu.r[SI] = sp.wrapping_add(4);
        Ok(())
    }
    #[inline]
    fn upop(&mut self) -> R<u32> {
        let v = self.eax();
        self.drop1()?;
        Ok(v)
    }
    #[inline]
    fn setv(&mut self, a: u32, v: u32) -> R {
        self.mem.write_u32(a, v)
    }
    #[inline]
    fn rpush(&mut self, v: u32) -> R {
        self.cpu.push32(&mut self.mem, v)
    }
    #[inline]
    fn rpop(&mut self) -> R<u32> {
        self.cpu.pop32(&self.mem)
    }
    #[inline]
    fn compiling(&self) -> bool {
        self.var(st::IS_COMPILING) != 0
    }
    #[inline]
    fn here(&self) -> u32 {
        self.var(st::H)
    }

    // ---- code generation (compile, literal, comma, align) -------------

    fn reserve_code(&self, n: u32) -> R<u32> {
        let h = self.here();
        if h.checked_add(n).is_none_or(|e| e > HTOP) {
            return Err(Fault::OutOfSpace("code"));
        }
        Ok(h)
    }

    fn emit_code(&mut self, bytes: &[u8]) -> R {
        let h = self.reserve_code(bytes.len() as u32)?;
        self.mem.write_bytes(h, bytes)?;
        self.setv(st::H, h + bytes.len() as u32)
    }

    /// `PROC compile`: `call rel32` to `xt`.
    fn compile_call(&mut self, xt: u32) -> R {
        let rel = xt.wrapping_sub(self.here().wrapping_add(5));
        let mut b = [0xe8, 0, 0, 0, 0];
        b[1..].copy_from_slice(&rel.to_le_bytes());
        self.emit_code(&b)
    }

    /// `PROC literal`: `lea esi,[esi-4]; mov [esi],eax; mov eax,imm32`.
    fn compile_literal(&mut self, v: u32) -> R {
        let mut b = [0x8d, 0x76, 0xfc, 0x89, 0x06, 0xb8, 0, 0, 0, 0];
        b[6..].copy_from_slice(&v.to_le_bytes());
        self.emit_code(&b)
    }

    /// `PROC comma` and friends: store TOS at `here`, advance by `n`.  Like
    /// the original, all four bytes are written whatever `n` is.
    fn comma_n(&mut self, n: u32) -> R {
        let h = self.reserve_code(4)?;
        self.mem.write_u32(h, self.eax())?;
        self.setv(st::H, h + n)?;
        self.drop1()
    }

    /// `PROC _align`: pad code space to 4 bytes with `nop`s.
    fn align_code(&mut self) -> R {
        let start = self.here();
        let mut h = start;
        while h & 3 != 0 {
            self.mem.write_u8(h, 0x90)?;
            h += 1;
        }
        if h != start {
            self.last_pad = (start, h);
        }
        self.setv(st::H, h)
    }

    /// `PROC store_here` (`here,`): copy `n` bytes from `src` to `here`.
    fn store_here(&mut self, src: u32, n: u32) -> R {
        let h = self.reserve_code(n)?;
        self.mem.move_forward(src, h, n)?;
        self.setv(st::H, h + n)
    }

    /// `PROC ssemi` (`;;`): compile `ret`, then turn a trailing
    /// `call rel32` into `jmp rel32` (tail call) unless the last word
    /// compiled was of a `notail` class; an indirect `call [abs]` becomes
    /// `jmp [abs]`.  The byte tests are the original's, including which
    /// offsets it inspects.
    ///
    /// Deliberate difference: if the last thing in code space is alignment
    /// padding (a header was made, e.g. by `:` executed inside a definition
    /// as in `reva.f`'s `::`), the five bytes before `ret` are not one
    /// instruction and the original would patch whatever byte happens to be
    /// there — harmless only when that byte is not `E8`/`FF`.  Layout here
    /// differs from the x86 build, so that coincidence must not be relied on.
    fn ssemi(&mut self) -> R {
        let h = self.reserve_code(1)?;
        self.mem.write_u8(h, 0xc3)?;
        self.setv(st::H, h + 1)?;
        let b = h.wrapping_sub(5);
        let (ps, pe) = self.last_pad;
        if pe == h && ps > b {
            return Ok(());
        }
        match self.mem.read_u8(b)? {
            0xe8 => {
                if self.var(st::SSEMI_DO_TAIL) == 0x90 {
                    self.mem.write_u8(b, 0xe9)?;
                    self.setv(st::H, h)?;
                }
            }
            0xff if self.mem.read_u8(b + 1)? == 0x15 => {
                self.mem.write_u8(b + 1, 0x25)?;
                self.setv(st::H, h)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn lbracket(&mut self) -> R {
        self.setv(st::DIDHEADER, 0)?;
        self.setv(st::IS_COMPILING, 0)
    }

    /// `PROC inlinesemi` (`;inline`): move the word's code into the
    /// dictionary (count byte, code, `ret`) and make it class `'inline`, so
    /// later uses copy the bytes instead of calling.
    fn inline_semi(&mut self) -> R {
        let h = self.here();
        let lasth = self.var(st::LASTH);
        let size = h.wrapping_sub(lasth);
        let mut d = self.var(st::D);
        if d.checked_add(size + 6).is_none_or(|e| e > DTOP) {
            return Err(Fault::OutOfSpace("dictionary"));
        }
        self.mem.write_u32(d, size)?;
        d += 1;
        let xt = d;
        self.mem.move_forward(lasth, d, size)?;
        d += size;
        self.mem.write_u8(d, 0xc3)?;
        self.setv(st::D, d + 1)?;
        self.setv(st::H, lasth)?;
        let entry = self.mem.read_u32(self.var(st::LAST))?;
        self.mem
            .write_u32(entry.wrapping_sub(4), Gate::IClass.addr())?;
        self.mem.write_u32(entry + 4, xt)?;
        self.lbracket()
    }

    /// `PROC __header` ( a n -- ): create a dictionary entry
    /// `[class][link][xt][count][name...]` for the word about to be compiled
    /// at `here`, remembering enough to undo it if compilation fails.
    fn header_default(&mut self, host: &mut dyn Host) -> R {
        self.setv(st::DIDHEADER, 1)?;
        self.align_code()?;
        let d = (self.var(st::D) + 3) & !3;
        self.setv(st::LASTH, self.here())?;
        self.setv(st::LASTD, d)?;
        let lastp = self.var(st::LAST);
        let prev = self.mem.read_u32(lastp)?;
        self.setv(st::LASTL, prev)?;
        let n = self.upop()?;
        let a = self.upop()?;
        if d.checked_add(13 + n).is_none_or(|e| e > DTOP) {
            let v = self.var(st::DICTGONE);
            if v != Gate::OsBye.addr() {
                return self.call_nested(host, v);
            }
            return Err(Fault::OutOfSpace("dictionary"));
        }
        let entry = d + 4;
        self.mem.write_u32(d, self.var(st::DEFAULT_CLASS))?;
        self.mem.write_u32(lastp, entry)?;
        self.mem.write_u32(entry, prev)?;
        self.mem.write_u32(entry + 4, self.here())?;
        self.mem.write_u8(entry + 8, n as u8)?;
        self.mem.move_forward(a, entry + 9, n)?;
        self.setv(st::D, entry + 9 + n)
    }

    // ---- parsing -------------------------------------------------------

    /// Run a DEFER vector that takes no arguments, skipping it when it still
    /// points at `noop` (the common case, and the cheap one).
    fn call_vector(&mut self, host: &mut dyn Host, cell: u32) -> R {
        let t = self.var(cell);
        if t == Gate::Noop.addr() {
            return Ok(());
        }
        self.call_nested(host, t)
    }

    /// `PROC parsews`: skip blanks (9..=32), return the next word.  Each
    /// newline crossed runs the `do_cr` vector (line counting in `include`).
    fn parsews(&mut self, host: &mut dyn Host) -> R<(u32, u32)> {
        let end = self.var(st::TP);
        let mut p = self.var(st::TIN);
        let blank = |c: u8| (9..=32).contains(&c);
        while p < end {
            let c = self.mem.read_u8(p)?;
            if c == b'\n' {
                p += 1;
                self.setv(st::TIN, p)?;
                self.call_vector(host, st::DO_CR)?;
                continue;
            }
            if !blank(c) {
                break;
            }
            p += 1;
        }
        let start = p;
        while p < end && !blank(self.mem.read_u8(p)?) {
            p += 1;
        }
        let len = p - start;
        if p < end {
            let nl = self.mem.read_u8(p)? == b'\n';
            p += 1;
            self.setv(st::TIN, p)?;
            if nl {
                self.call_vector(host, st::DO_CR)?;
            }
        } else {
            self.setv(st::TIN, p)?;
        }
        Ok((start, len))
    }

    /// `PROC parse` ( c -- a n ): up to the next `c` (exclusive).
    fn parse(&mut self) -> R {
        let c = self.eax() as u8;
        let (start, end) = (self.var(st::TIN), self.var(st::TP));
        let mut p = start;
        while p < end && self.mem.read_u8(p)? != c {
            p += 1;
        }
        let len = p - start;
        self.setv(st::TIN, if p < end { p + 1 } else { end })?;
        self.set_eax(start);
        self.upsh(len)
    }

    /// `PROC parse_with_escape` (`parse/`) ( c -- a n ): like `parse`, but
    /// `\x` stands for `x`; the result is copied to the parse buffer.
    fn parse_escaped(&mut self) -> R {
        let term = self.eax() as u8;
        let end = self.var(st::TP);
        let mut p = self.var(st::TIN);
        let mut out = PARSE_BUF;
        while p < end && out < PARSE_BUF + PARSE_BUF_SIZE {
            let mut c = self.mem.read_u8(p)?;
            p += 1;
            if c == b'\\' && p < end {
                c = self.mem.read_u8(p)?;
                p += 1;
            } else if c == term {
                break;
            }
            self.mem.write_u8(out, c)?;
            out += 1;
        }
        self.setv(st::TIN, p)?;
        self.set_eax(PARSE_BUF);
        self.upsh(out - PARSE_BUF)
    }

    /// `PROC single`: `[-][#$%&]digits` or `'c`, in the current base.
    /// Overflow is an error, as in the original (`jo .err`).
    fn parse_single(&self, a: u32, n: u32) -> R<Option<u32>> {
        if n == 0 {
            return Ok(None);
        }
        let s = self.mem.slice(a, n)?;
        let mut base = self.var(st::BASE);
        let mut i = 0;
        let neg = s[0] == b'-';
        if neg {
            i = 1;
            if i == s.len() {
                return Ok(None);
            }
        }
        let sign = |v: u32| if neg { v.wrapping_neg() } else { v };
        if (b'#'..=b'\'').contains(&s[i]) {
            let k = (s[i] - b'#') as usize;
            if k == 4 {
                return Ok(s.get(i + 1).map(|&c| sign(c as u32)));
            }
            base = BASES[k];
            i += 1;
            if i == s.len() {
                return Ok(None);
            }
        }
        let mut acc = 0u32;
        for &c in &s[i..] {
            let Some(d) = digit_value(c as u32, base) else {
                return Ok(None);
            };
            let full = acc as u64 * base as u64;
            if full > u32::MAX as u64 {
                return Ok(None);
            }
            let (sum, overflow) = (full as u32 as i32).overflowing_add(d as i32);
            if overflow {
                return Ok(None);
            }
            acc = sum as u32;
        }
        Ok(Some(sign(acc)))
    }

    /// `PROC double`: like `single` with a trailing `L`; `.` and `,` are
    /// ignored as digit separators; 64-bit wrap-around arithmetic.
    fn parse_double(&self, a: u32, n: u32) -> R<Option<(u32, u32)>> {
        if n < 2 {
            return Ok(None);
        }
        let s = self.mem.slice(a, n)?;
        if s[s.len() - 1] != b'L' {
            return Ok(None);
        }
        let s = &s[..s.len() - 1];
        let mut base = self.var(st::BASE) as u64;
        let mut i = 0;
        let neg = s[0] == b'-';
        if neg {
            i = 1;
            if i == s.len() {
                return Ok(None);
            }
        }
        if (b'#'..=b'&').contains(&s[i]) {
            base = BASES[(s[i] - b'#') as usize] as u64;
            i += 1;
            if i == s.len() {
                return Ok(None);
            }
        }
        let mut acc = 0u64;
        for &c in &s[i..] {
            if c == b'.' || c == b',' {
                continue;
            }
            let Some(d) = digit_value(c as u32, base as u32) else {
                return Ok(None);
            };
            acc = acc.wrapping_mul(base).wrapping_add(d as u64);
        }
        if neg {
            acc = acc.wrapping_neg();
        }
        Ok(Some((acc as u32, (acc >> 32) as u32)))
    }

    /// `PROC _printr` (`(.r)`) ( n width -- a n ): format `n` in `base`
    /// (signed only in base 10), right-aligned with `padchar`.
    fn print_r(&mut self) -> R {
        let width = self.upop()? & 255;
        let n = self.eax();
        let base = self.var(st::BASE);
        if base < 2 {
            return Err(Fault::DivideError { eip: self.cpu.eip });
        }
        let neg = base == 10 && (n as i32) < 0;
        let mut v = if neg { n.wrapping_neg() } else { n };
        let mut p = NPAD;
        loop {
            let mut c = (v % base) as u8 + b'0';
            if c > b'9' {
                c += 7;
            }
            v /= base;
            p -= 1;
            self.mem.write_u8(p, c)?;
            if v == 0 {
                break;
            }
        }
        if neg {
            p -= 1;
            self.mem.write_u8(p, b'-')?;
        }
        let mut len = NPAD - p;
        if width > len {
            let pad = self.var(st::PADCHAR) as u8;
            for _ in len..width {
                p -= 1;
                self.mem.write_u8(p, pad)?;
            }
            len = width;
        }
        self.set_eax(p);
        self.upsh(len)
    }

    fn zlen(&self, a: u32) -> R<u32> {
        let mut p = a;
        while self.mem.read_u8(p)? != 0 {
            p += 1;
        }
        Ok(p - a)
    }

    /// `compare_common` + `PROC compare`/`comparei` ( a1 n1 a2 n2 -- r ).
    fn compare(&mut self, fold: bool) -> R {
        let n2 = self.upop()?;
        let a2 = self.upop()?;
        let n1 = self.upop()?;
        let a1 = self.eax();
        let r: i32 = if n1 == 0 || a1 == 0 || n2 == 0 || a2 == 0 {
            match (n1 as i32).cmp(&(n2 as i32)) {
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
                std::cmp::Ordering::Less => -1,
            }
        } else {
            let len = (n1 as i32).min(n2 as i32) as u32;
            let (s1, s2) = (self.mem.slice(a1, len)?, self.mem.slice(a2, len)?);
            let f = |c: u8| if fold { c.to_ascii_lowercase() } else { c };
            match s1.iter().zip(s2).find(|(x, y)| f(**x) != f(**y)) {
                Some((x, y)) => f(*x).wrapping_sub(f(*y)) as i8 as i32,
                None => n1.wrapping_sub(n2) as i32,
            }
        };
        self.set_eax(r as u32);
        Ok(())
    }

    // ---- lists -----------------------------------------------------------

    /// `PROC link` ( dataptr listptr -- ): push a node `[prior][dataptr]`
    /// built at `here` onto the front of the list.
    fn link(&mut self) -> R {
        let listptr = self.upop()?;
        let data = self.upop()?;
        let h = self.reserve_code(8)?;
        let prior = self.mem.read_u32(listptr)?;
        self.mem.write_u32(listptr, h)?;
        self.mem.write_u32(h, prior)?;
        self.mem.write_u32(h + 4, data)?;
        self.setv(st::H, h + 8)
    }

    /// `PROC link2` (`-link`): append at the end of the list instead.
    fn link_end(&mut self) -> R {
        let mut p = self.eax();
        loop {
            let next = self.mem.read_u32(p)?;
            if next == 0 {
                break;
            }
            p = next;
        }
        self.set_eax(p);
        self.link()
    }

    /// `PROC iterate`: call `xt` with each node's data address until it
    /// returns 0.  Runs as nested calls.
    fn iterate_list(&mut self, host: &mut dyn Host, xt: u32, list: u32) -> R {
        if xt == 0 {
            return Ok(());
        }
        let mut node = list;
        loop {
            node = self.mem.read_u32(node)?;
            if node == 0 {
                return Ok(());
            }
            self.upsh(node + 4)?;
            self.call_nested(host, xt)?;
            if self.upop()? == 0 {
                return Ok(());
            }
        }
    }

    // ---- output via (possibly redefined) vectors -------------------------

    fn call_type(&mut self, host: &mut dyn Host) -> R {
        let t = self.var(st::TYPE);
        if t != Gate::OsType.addr() {
            return self.call_nested(host, t);
        }
        let n = self.upop()?;
        let a = self.upop()?;
        if n != 0 {
            host.write(self.mem.slice(a, n)?);
        }
        Ok(())
    }

    fn call_emit(&mut self, host: &mut dyn Host, c: u8) -> R {
        let t = self.var(st::EMIT);
        if t != Gate::OsEmit.addr() {
            self.upsh(c as u32)?;
            return self.call_nested(host, t);
        }
        host.write(&[c]);
        Ok(())
    }

    // ---- files and heap ------------------------------------------------

    /// `PROC _slurp` ( a n -- a' n' ): read a whole file into the heap.
    /// Failure leaves ( 0 0 ), sets `ioerr` and CF.
    fn slurp(&mut self, host: &mut dyn Host) -> R<bool> {
        let n = self.eax();
        let a = self.nos()?;
        let path = String::from_utf8_lossy(self.mem.slice(a, n)?).into_owned();
        let loaded = match host.read_file(&path) {
            Some(bytes) => match self.heap.alloc(bytes.len() as u32) {
                Some(p) => {
                    self.mem.write_bytes(p, &bytes)?;
                    Some((p, bytes.len() as u32))
                }
                None => None,
            },
            None => None,
        };
        let (p, len) = loaded.unwrap_or((0, 0));
        self.set_nos(p)?;
        self.set_eax(len);
        self.cpu.cf = loaded.is_none();
        self.setv(st::IOR, loaded.is_none() as u32)?;
        Ok(loaded.is_some())
    }

    fn resize(&mut self) -> R {
        let n = self.upop()?;
        let a = self.eax();
        let new = if a == 0 {
            self.heap.alloc(n)
        } else {
            match self.heap.size_of(a) {
                Some(old) => {
                    let new = self.heap.alloc(n);
                    if let Some(p) = new {
                        self.mem.move_forward(a, p, old.min(n))?;
                        self.heap.free(a);
                    }
                    new
                }
                None => None,
            }
        };
        self.set_eax(new.unwrap_or(0));
        self.setv(st::IOR, new.is_none() as u32)
    }

    // ---- the outer interpreter ------------------------------------------

    /// One key of `query`'s line editor.  `None` means "keep reading".
    fn query_char(&mut self, c: u32) -> R<Option<Next>> {
        let tp = self.var(st::TP);
        if c == EOF_KEY {
            // Deviation: newcore exited at once, losing a final line that
            // had no newline.  Interpret that line first; the next EOF exits.
            if tp > TIB {
                return Ok(Some(Next::Jump(Gate::InterpWord.addr())));
            }
            self.upsh(0)?;
            return Ok(Some(Next::Jump(Gate::Bye.addr())));
        }
        if c == u32::MAX {
            return Ok(Some(Next::Jump(Gate::Interpret.addr())));
        }
        match c as u8 {
            b'\n' => return Ok(Some(Next::Jump(Gate::InterpWord.addr()))),
            b'\r' => {}
            8 => self.setv(st::TP, if tp > TIB { tp - 1 } else { TIB })?,
            ch => {
                self.mem.write_u8(tp, if ch == b'\t' { b' ' } else { ch })?;
                self.setv(st::TP, tp + 1)?;
                if tp + 1 >= TIB_TOP {
                    return Ok(Some(Next::Jump(Gate::InterpWord.addr())));
                }
            }
        }
        Ok(None)
    }

    /// `interpret`'s `.afterok`: `ecx` is the token kind + 1 from the
    /// tokenizer (1 = dictionary entry, 2 = single, 3 = double).
    fn after_ok(&mut self, ecx: u32) -> R<Next> {
        let kind = ecx.wrapping_sub(1);
        if kind == 0 {
            let entry = self.eax();
            let class = self.mem.read_u32(entry.wrapping_sub(4))?;
            self.cpu.r[EDX as usize] = entry;
            self.cpu.r[EDI as usize] = class;
            self.set_eax(self.mem.read_u32(entry.wrapping_add(4))?);
            return Ok(Next::Call(class, Gate::InterpWord));
        }
        if self.compiling() {
            match kind {
                1 => {
                    let v = self.upop()?;
                    self.compile_literal(v)?;
                }
                2 => {
                    let hi = self.upop()?;
                    let lo = self.upop()?;
                    self.compile_literal(lo)?;
                    self.compile_literal(hi)?;
                }
                _ => {}
            }
        }
        Ok(Next::Jump(Gate::InterpWord.addr()))
    }

    fn mclass(&mut self) -> R<Next> {
        let xt = self.upop()?;
        self.cpu.r[EBX as usize] = xt;
        Ok(Next::Jump(xt))
    }

    fn not_implemented(g: Gate) -> Fault {
        Fault::NotImplemented(match g {
            Gate::MakeExe => "makeexe",
            Gate::Syscall => "syscall",
            Gate::LzPack => ">lz",
            Gate::LzDepack => "lz>",
            Gate::LzMax => "lzmax",
            Gate::LoadLib => "(lib)",
            Gate::UnloadLib => "(-lib)",
            Gate::CallFn => "(call)",
            Gate::OsFunc => "(func)",
            Gate::OpenR => "open/r",
            Gate::OpenRw => "open/rw",
            Gate::Creat => "creat",
            Gate::Close => "close",
            Gate::Read => "read",
            Gate::Write => "write",
            Gate::Fsize => "fsize",
            Gate::Save => "(save)",
            _ => g.name(),
        })
    }

    // ---- the dispatcher ------------------------------------------------

    pub(crate) fn gate(&mut self, g: Gate, host: &mut dyn Host) -> R<Next> {
        use Gate::*;
        use Next::{Call, Jump, Ret};
        match g {
            // --- classes: called with EAX = xt, EDX = dictionary entry ---
            FClass | FClassNotail => {
                self.setv(st::SSEMI_DO_TAIL, if g == FClass { 0x90 } else { 0xc3 })?;
                if self.compiling() {
                    let xt = self.upop()?;
                    self.compile_call(xt)?;
                    Ok(Ret)
                } else {
                    self.mclass()
                }
            }
            MClass => self.mclass(),
            MClassNotail => {
                self.setv(st::SSEMI_DO_TAIL, 0xc3)?;
                self.mclass()
            }
            IClass => {
                self.setv(st::SSEMI_DO_TAIL, 0xc3)?;
                if !self.compiling() {
                    return self.mclass();
                }
                let xt = self.upop()?;
                let n = self.mem.read_u8(xt.wrapping_sub(1))? as u32;
                self.cpu.r[EDX as usize] = xt;
                self.store_here(xt, n)?;
                Ok(Ret)
            }
            CClass => {
                // `cclass` has no `ret`: it falls through into `vclass`, so
                // while compiling the constant's value becomes a literal.
                let v = self.mem.read_u32(self.eax())?;
                self.set_eax(v);
                if self.compiling() {
                    let v = self.upop()?;
                    self.compile_literal(v)?;
                }
                Ok(Ret)
            }
            VClass => {
                if self.compiling() {
                    let v = self.upop()?;
                    self.compile_literal(v)?;
                }
                Ok(Ret)
            }
            ValClass => {
                if self.compiling() {
                    let v = self.upop()?;
                    self.compile_literal(v)?;
                    self.compile_call(CClass.addr())?;
                } else {
                    let v = self.mem.read_u32(self.eax())?;
                    self.set_eax(v);
                }
                Ok(Ret)
            }
            DoesClass => {
                // xt-4 holds the address of the code after `does>`.
                self.dup()?;
                let code = self.mem.read_u32(self.eax().wrapping_sub(4))?;
                self.set_eax(code);
                if !self.compiling() {
                    return self.mclass();
                }
                let code = self.upop()?;
                let xt = self.upop()?;
                self.compile_literal(xt)?;
                self.compile_call(code)?;
                Ok(Ret)
            }
            DClass => {
                if self.compiling() {
                    // call dword [xt]
                    let xt = self.upop()?;
                    let mut b = [0xff, 0x15, 0, 0, 0, 0];
                    b[2..].copy_from_slice(&xt.to_le_bytes());
                    self.emit_code(&b)?;
                    Ok(Ret)
                } else {
                    let cell = self.upop()?;
                    self.cpu.r[EBX as usize] = cell;
                    Ok(Jump(self.mem.read_u32(cell)?))
                }
            }

            // --- default DEFER targets ---
            Noop => Ok(Ret),
            OsEmit => {
                let c = self.upop()?;
                host.write(&[c as u8]);
                Ok(Ret)
            }
            OsType => {
                // The original passed `n` to write(2); a negative count (as
                // `srcstr` yields at the keyboard: `src` is 0 there) made the
                // kernel refuse with EFAULT and print nothing.  Same here.
                let n = self.upop()?;
                let a = self.upop()?;
                if n != 0 && (n as i32) > 0 {
                    host.write(self.mem.slice(a, n)?);
                }
                Ok(Ret)
            }
            OsKey => match self.input.pop_front() {
                Some(b) => {
                    self.upsh(b as u32)?;
                    Ok(Ret)
                }
                None if self.input_closed => {
                    self.upsh(EOF_KEY)?;
                    Ok(Ret)
                }
                None => Ok(Next::NeedInput),
            },
            OsBye => Ok(Next::Halt(self.eax() as i32)),
            Find => {
                // ( a n -- entry | a n 0 )
                let n = self.upop()?;
                let a = self.eax();
                let name = self.mem.slice(a, n)?.to_vec();
                match self.find_entry(&name) {
                    Some(e) => self.set_eax(e),
                    None => {
                        self.upsh(n)?;
                        self.upsh(0)?;
                    }
                }
                Ok(Ret)
            }
            WordNotFound => {
                // ( a n -- 0 ): print "name?" and give up on the token.
                self.call_type(host)?;
                self.call_emit(host, b'?')?;
                self.call_emit(host, b'\n')?;
                self.upsh(0)?;
                Ok(Ret)
            }
            HeaderDefault => {
                self.header_default(host)?;
                Ok(Ret)
            }
            Hello => {
                // `_hello`: include argv[1] if given, then run the REPL.
                if self.var(st::ARGC) > 1 {
                    let p = self.mem.read_u32(self.var(st::ARGV) + 4)?;
                    let n = self.zlen(p)?;
                    self.upsh(p)?;
                    self.upsh(n)?;
                    Ok(Call(Include2.addr(), Repl))
                } else {
                    Ok(Jump(Repl.addr()))
                }
            }

            // --- startup ---
            Cold => {
                self.cpu.r[SI] = S0;
                self.set_eax(0);
                self.setv(st::RP0_SAVED, self.cpu.r[ESP as usize])?;
                self.iterate_list(host, OnStartThunk.addr(), st::START_LIST)?;
                Ok(Call(self.var(st::APPSTART), ColdDone))
            }
            BootRestart => {
                // reva.f's last step saved a native executable; running that
                // executable started with fresh stacks and no source file,
                // ran the `onstartup` list (which allocates `scratch`, `pad`
                // ...) and then `appstart`.  Do the same in place: drop the
                // boot file from argv, abandon the include frames, restart.
                // TOS is reva.f's interpreter loop, for fault recovery.
                let repl = self.upop()?;
                self.setv(st::REPL_ENTRY, repl)?;
                self.setv(st::ARGC, 1)?;
                self.setv(st::SOURCE, 0)?;
                self.setv(st::TP, TIB)?;
                self.setv(st::TIN, TIB)?;
                self.setv(st::IS_COMPILING, 0)?;
                self.cpu.r[ESP as usize] = RP0;
                Ok(Jump(Cold.addr()))
            }
            ColdDone => {
                self.set_eax(u32::MAX);
                Ok(Jump(Bye.addr()))
            }
            Repl => Ok(Call(Interpret.addr(), Repl)),
            OnStartThunk => Ok(Jump(self.mem.read_u32(self.eax())?)),

            // --- interpret / query ---
            Interpret => {
                let src = self.var(st::SOURCE);
                if src != 0 {
                    if self.var(st::TIN) >= src {
                        return Ok(Ret);
                    }
                    self.setv(st::TP, src)?;
                    return Ok(Jump(InterpWord.addr()));
                }
                self.call_vector(host, st::PROMPT)?;
                self.setv(st::TP, TIB)?;
                self.setv(st::TIN, TIB)?;
                Ok(Jump(QueryKey.addr()))
            }
            QueryKey => {
                let key = self.var(st::KEY);
                if key != OsKey.addr() {
                    return Ok(Call(key, QueryKeyGot));
                }
                loop {
                    let c = match self.input.pop_front() {
                        Some(b) => b as u32,
                        None if self.input_closed => EOF_KEY,
                        None => return Ok(Next::NeedInput),
                    };
                    if let Some(next) = self.query_char(c)? {
                        return Ok(next);
                    }
                }
            }
            QueryKeyGot => {
                let c = self.upop()?;
                Ok(self.query_char(c)?.unwrap_or(Jump(QueryKey.addr())))
            }
            InterpWord => {
                if self.cpu.r[SI] > S0 {
                    return Err(Fault::StackUnderflow);
                }
                let (a, n) = self.parsews(host)?;
                if n == 0 {
                    return Ok(Jump(Interpret.addr()));
                }
                self.upsh(a)?;
                self.upsh(n)?;
                Ok(Call(self.var(st::FINDINTERNAL), InterpFound))
            }
            InterpFound => {
                if self.eax() != 0 {
                    return self.after_ok(1);
                }
                self.drop1()?;
                let (a, n) = (self.nos()?, self.eax());
                if let Some(v) = self.parse_single(a, n)? {
                    self.drop1()?;
                    self.set_eax(v);
                    return self.after_ok(2);
                }
                if let Some((lo, hi)) = self.parse_double(a, n)? {
                    self.set_nos(lo)?;
                    self.set_eax(hi);
                    return self.after_ok(3);
                }
                Ok(Call(self.var(st::IS_WORD), InterpIsWord))
            }
            InterpIsWord => {
                let kind = self.upop()?;
                if kind != 0 {
                    return self.after_ok(kind);
                }
                // `.fail`: abandon a half-made definition, leave interpret.
                self.rollback_definition()?;
                Ok(Ret)
            }
            Eval => {
                let n = self.upop()?;
                let a = self.upop()?;
                self.rpush(self.var(st::SOURCE))?;
                self.rpush(self.var(st::TIN))?;
                self.rpush(self.var(st::TP))?;
                self.setv(st::SOURCE, a.wrapping_add(n))?;
                self.setv(st::TIN, a)?;
                Ok(Call(Interpret.addr(), EvalDone))
            }
            EvalDone => {
                let tp = self.rpop()?;
                let tin = self.rpop()?;
                let src = self.rpop()?;
                self.setv(st::TP, tp)?;
                self.setv(st::TIN, tin)?;
                self.setv(st::SOURCE, src)?;
                Ok(Ret)
            }
            Include => {
                let (a, n) = self.parsews(host)?;
                self.upsh(a)?;
                self.upsh(n)?;
                Ok(Jump(Include2.addr()))
            }
            Include2 => {
                // `__include` ( a n -- ): slurp, eval, free.  As in the
                // original, the data stack is restored afterwards.
                if !self.slurp(host)? {
                    self.drop1()?;
                    self.drop1()?;
                    self.setv(st::IOR, 1)?;
                    return Ok(Ret);
                }
                self.rpush(self.nos()?)?;
                self.rpush(self.cpu.r[SI])?;
                self.rpush(self.eax())?;
                Ok(Call(Eval.addr(), IncludeDone))
            }
            IncludeDone => {
                let eax = self.rpop()?;
                self.cpu.r[SI] = self.rpop()?;
                self.set_eax(eax);
                self.drop1()?;
                let ptr = self.rpop()?;
                self.heap.free(ptr);
                self.drop1()?;
                self.setv(st::IOR, 0)?;
                Ok(Ret)
            }

            // --- catch / throw: ESP-based frames, exactly as in newcore ---
            Catch => {
                self.rpush(self.cpu.r[SI])?;
                self.rpush(self.var(st::HANDLER))?;
                self.setv(st::HANDLER, self.cpu.r[ESP as usize])?;
                let xt = self.upop()?;
                self.cpu.r[EBX as usize] = xt;
                Ok(Call(xt, CatchDone))
            }
            CatchDone => {
                let h = self.rpop()?;
                self.setv(st::HANDLER, h)?;
                self.rpop()?;
                self.upsh(0)?;
                Ok(Ret)
            }
            Throw => {
                let h = self.var(st::HANDLER);
                if self.eax() == 0 || h == 0 {
                    self.drop1()?;
                    return Ok(Ret);
                }
                self.cpu.r[ESP as usize] = h;
                let prev = self.rpop()?;
                self.setv(st::HANDLER, prev)?;
                self.cpu.r[SI] = self.rpop()?;
                Ok(Ret)
            }
            NestedReturn => {
                if self.nested == 0 {
                    return Err(Fault::BadGate { addr: g.addr() });
                }
                Ok(Next::NestedReturn)
            }

            // --- defining words ---
            Header => {
                let (a, n) = self.parsews(host)?;
                self.upsh(a)?;
                self.upsh(n)?;
                Ok(Jump(self.var(st::HEADER_)))
            }
            Colon => {
                let (a, n) = self.parsews(host)?;
                self.upsh(a)?;
                self.upsh(n)?;
                Ok(Call(self.var(st::HEADER_), RBracket))
            }
            Tick => {
                let (a, n) = self.parsews(host)?;
                self.upsh(a)?;
                self.upsh(n)?;
                Ok(Call(self.var(st::FINDINTERNAL), TickDone))
            }
            TickDone => {
                if self.eax() == 0 {
                    self.drop1()?;
                    self.drop1()?;
                    self.set_eax(0);
                } else {
                    let xt = self.mem.read_u32(self.eax() + 4)?;
                    self.set_eax(xt);
                }
                Ok(Ret)
            }
            Variable => {
                self.upsh(0)?;
                Ok(Jump(Variable2.addr()))
            }
            Variable2 => {
                self.rpush(self.var(st::DEFAULT_CLASS))?;
                self.setv(st::DEFAULT_CLASS, VClass.addr())?;
                let (a, n) = self.parsews(host)?;
                self.upsh(a)?;
                self.upsh(n)?;
                Ok(Call(self.var(st::HEADER_), Variable2Done))
            }
            Variable2Done => {
                let dc = self.rpop()?;
                self.setv(st::DEFAULT_CLASS, dc)?;
                self.comma_n(4)?;
                Ok(Ret)
            }
            Literal => {
                let v = self.upop()?;
                self.compile_literal(v)?;
                Ok(Ret)
            }
            SSemi => {
                self.ssemi()?;
                Ok(Ret)
            }
            Semi => {
                self.ssemi()?;
                self.lbracket()?;
                Ok(Ret)
            }
            InlineSemi => {
                self.inline_semi()?;
                Ok(Ret)
            }
            LBracket => {
                self.lbracket()?;
                Ok(Ret)
            }
            RBracket => {
                let s = self.var(st::IS_COMPILING);
                self.setv(st::IS_COMPILING, s.wrapping_add(1))?;
                Ok(Ret)
            }
            Compile => {
                let xt = self.upop()?;
                self.compile_call(xt)?;
                Ok(Ret)
            }
            Align => {
                self.align_code()?;
                Ok(Ret)
            }
            StoreHere => {
                let n = self.upop()?;
                let a = self.upop()?;
                self.store_here(a, n)?;
                Ok(Ret)
            }
            Comma | Comma3 | Comma2 | Comma1 => {
                let n = match g {
                    Comma => 4,
                    Comma3 => 3,
                    Comma2 => 2,
                    _ => 1,
                };
                self.comma_n(n)?;
                Ok(Ret)
            }

            // --- parsing and numbers ---
            Parsews => {
                let (a, n) = self.parsews(host)?;
                self.upsh(a)?;
                self.upsh(n)?;
                Ok(Ret)
            }
            Parse => {
                self.parse()?;
                Ok(Ret)
            }
            ParseEsc => {
                self.parse_escaped()?;
                Ok(Ret)
            }
            Single => {
                let (a, n) = (self.nos()?, self.eax());
                match self.parse_single(a, n)? {
                    Some(v) => {
                        self.drop1()?;
                        self.set_eax(v);
                        self.upsh(u32::MAX)?;
                    }
                    None => self.upsh(0)?,
                }
                Ok(Ret)
            }
            Double => {
                let (a, n) = (self.nos()?, self.eax());
                match self.parse_double(a, n)? {
                    Some((lo, hi)) => {
                        self.set_nos(lo)?;
                        self.set_eax(hi);
                        self.upsh(u32::MAX)?;
                    }
                    None => self.upsh(0)?,
                }
                Ok(Ret)
            }
            FromDigit => {
                let v = digit_value(self.eax(), self.var(st::BASE)).unwrap_or(u32::MAX);
                self.set_eax(v);
                Ok(Ret)
            }
            ToDigit => {
                let n = self.eax();
                let c = if n < 16 {
                    DIGITS[n as usize] as u32
                } else {
                    let base = self.var(st::BASE);
                    if base == 0 {
                        return Err(Fault::DivideError { eip: g.addr() });
                    }
                    let c = n % base + 0x30;
                    if c > 0x39 {
                        c + 39
                    } else {
                        c
                    }
                };
                self.set_eax(c);
                Ok(Ret)
            }
            PrintR => {
                self.print_r()?;
                Ok(Ret)
            }

            // --- memory and strings ---
            TwoDup => {
                let a = self.nos()?;
                let b = self.eax();
                let sp = self.cpu.r[SI].wrapping_sub(8);
                self.mem.write_u32(sp, a)?;
                self.mem.write_u32(sp + 4, b)?;
                self.cpu.r[SI] = sp;
                Ok(Ret)
            }
            Move => {
                let n = self.upop()?;
                let dst = self.upop()?;
                let src = self.upop()?;
                self.mem.move_forward(src, dst, n)?;
                Ok(Ret)
            }
            Fill => {
                let c = self.upop()?;
                let n = self.upop()?;
                let a = self.upop()?;
                self.mem.fill(a, n, c as u8)?;
                Ok(Ret)
            }
            Place | PlusPlace => {
                let buf = self.upop()?;
                let len = self.upop()?;
                let s = self.upop()?;
                if buf == 0 {
                    return Ok(Ret);
                }
                let dest = if g == Place {
                    if s == 0 {
                        self.mem.write_u8(buf, 0)?;
                        return Ok(Ret);
                    }
                    self.mem.write_u8(buf, len as u8)?;
                    buf + 1
                } else {
                    let cnt = self.mem.read_u8(buf)?;
                    self.mem.write_u8(buf, cnt.wrapping_add(len as u8))?;
                    buf + cnt as u32 + 1
                };
                self.mem.move_forward(s, dest, len)?;
                self.mem.write_u8(dest + len, 0)?;
                Ok(Ret)
            }
            Zt => {
                let len = self.upop()?;
                let s = self.eax();
                if s != 0 {
                    self.mem.write_u8(s.wrapping_add(len), 0)?;
                }
                Ok(Ret)
            }
            Zcount => {
                let s = self.eax();
                let n = if s == 0 { 0 } else { self.zlen(s)? };
                self.upsh(n)?;
                Ok(Ret)
            }
            Cmp | Cmpi => {
                self.compare(g == Cmpi)?;
                Ok(Ret)
            }
            FnvHash => {
                let n = self.upop()?;
                let a = self.eax();
                let mut h: u32 = 2_166_136_261;
                for &b in self.mem.slice(a, n)? {
                    h = (h ^ b as u32).wrapping_mul(16_777_619);
                }
                self.set_eax(h);
                Ok(Ret)
            }
            Allocate => {
                let a = self.heap.alloc(self.eax()).unwrap_or(0);
                self.set_eax(a);
                self.setv(st::IOR, (a == 0) as u32)?;
                Ok(Ret)
            }
            Free => {
                let a = self.upop()?;
                self.heap.free(a);
                self.setv(st::IOR, 0)?;
                Ok(Ret)
            }
            Resize => {
                self.resize()?;
                Ok(Ret)
            }
            Slurp => {
                self.slurp(host)?;
                Ok(Ret)
            }
            SetExtra => {
                let n = self.upop()?;
                let a = self.upop()?;
                self.setv(st::EXTRA_SIZE, n)?;
                self.setv(st::EXTRA_DATA, a)?;
                Ok(Ret)
            }
            GetExtra => {
                self.upsh(self.var(st::EXTRA_DATA))?;
                self.upsh(self.var(st::EXTRA_SIZE))?;
                Ok(Ret)
            }

            // --- lists and handlers ---
            Link => {
                self.link()?;
                Ok(Ret)
            }
            Link2 => {
                self.link_end()?;
                Ok(Ret)
            }
            OnExit => {
                self.upsh(st::EXIT_LIST)?;
                self.link()?;
                Ok(Ret)
            }
            OnStartup => {
                self.upsh(st::START_LIST)?;
                self.link_end()?;
                Ok(Ret)
            }
            Iterate => {
                let list = self.upop()?;
                let xt = self.upop()?;
                self.iterate_list(host, xt, list)?;
                Ok(Ret)
            }
            Reset => {
                self.cpu.r[SI] = S0;
                self.set_eax(0);
                self.setv(st::RP0_SAVED, self.cpu.r[ESP as usize])?;
                Ok(Ret)
            }
            Bye => {
                // run `onexit` handlers, then exit with TOS as the status
                let code = self.eax();
                self.iterate_list(host, OnStartThunk.addr(), st::EXIT_LIST)?;
                Ok(Next::Halt(code as i32))
            }

            // --- OS services not yet provided by the portable core ---
            // --- portable OS layer (ffi.rs) ---
            LoadLib => self.ffi_loadlib().map(|()| Ret),
            UnloadLib => self.ffi_unloadlib().map(|()| Ret),
            OsFunc => self.ffi_func().map(|()| Ret),
            CallFn => self.ffi_call(host).map(|()| Ret),
            Syscall => self.ffi_syscall(host).map(|()| Ret),
            HostGetenv | HostPutenv | HostLocaltime => {
                Err(Fault::NotImplemented("a C function called without (call)"))
            }

            MakeExe | LzPack | LzDepack | LzMax | OpenR | OpenRw | Creat | Close | Read | Write
            | Fsize | Save => Err(Self::not_implemented(g)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gates_fit_in_their_region() {
        assert!(Gate::ALL.len() as u32 * GATE_STRIDE <= GATE_END - GATE_BASE);
        for (i, g) in Gate::ALL.iter().enumerate() {
            assert_eq!(*g as usize, i);
            assert_eq!(Gate::from_addr(g.addr()), Some(*g));
        }
        assert_eq!(Gate::from_addr(GATE_BASE + 1), None);
        assert_eq!(Gate::from_addr(GATE_END), None);
    }

    #[test]
    fn digits() {
        assert_eq!(digit_value(b'7' as u32, 10), Some(7));
        assert_eq!(digit_value(b'f' as u32, 16), Some(15));
        assert_eq!(digit_value(b'F' as u32, 16), Some(15));
        assert_eq!(digit_value(b'9' as u32, 8), None);
        assert_eq!(digit_value(b'@' as u32, 36), None);
    }
}
