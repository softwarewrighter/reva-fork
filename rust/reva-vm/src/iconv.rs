//! # The virtual library `libiconv`
//!
//! `lib/string/iconv` binds `libiconv_open`, `libiconv`, `libiconv_close`
//! and the data symbol `_libiconv_version` (IUP's library loads it first,
//! so GUI programs stopped here).  iconv is pure computation, so the VM
//! does it itself, on every host, with no dependency (charset.rs).
//!
//! `iconv`'s contract moves pointers: `( desc **in *inleft **out *outleft )`
//! are VM addresses of cells, which this reads and advances in VM memory as
//! C's iconv does, so the Forth binding runs unchanged.  Failures return
//! `(size_t)-1` with `errno` set, as in C: E2BIG (output full), EILSEQ (a
//! byte sequence that is not a character, or a character the target
//! cannot hold), EINVAL (input ends inside a character; or an encoding
//! iconv_open does not know), EBADF (a descriptor not open).

use crate::charset::{self, Bad, Enc};
use crate::fault::Fault;
use crate::gates::Gate;
use crate::machine::{Machine, st};

/// Handle `(lib)` returns for this library ("ICNV").
pub const ICONV_HANDLE: u32 = 0x4943_4e56;

/// Library names `lib/string/iconv` uses per OS.
pub const ICONV_NAMES: &[&str] = &["libiconv.so", "libiconv.dylib", "libiconv2.dll", "libiconv"];

/// What `_libiconv_version` reads as: libiconv 1.16's (0x0110).
pub const VERSION: u32 = 0x0110;

const FAIL: u32 = u32::MAX;
const E2BIG: u32 = 7;
const EBADF: u32 = 9;
const EINVAL: u32 = 22;
const EILSEQ: u32 = 84;

/// The gate for a function name, or the data cell for a data symbol.
pub fn lookup(name: &str) -> Option<u32> {
    match name {
        "libiconv_open" | "iconv_open" => Some(Gate::IcOpen.addr()),
        "libiconv" | "iconv" => Some(Gate::Iconv.addr()),
        "libiconv_close" | "iconv_close" => Some(Gate::IcClose.addr()),
        "_libiconv_version" => Some(st::ICONV_VERSION),
        _ => None,
    }
}

impl Machine {
    /// `(call)` of an iconv function: `Some(result)`, or `None` if `g` is
    /// not one of them.
    pub(crate) fn iconv_call(&mut self, g: Gate, args: &[u32]) -> Result<Option<u32>, Fault> {
        let a = |i: usize| args.get(i).copied().unwrap_or(0);
        Ok(Some(match g {
            Gate::IcOpen => self.iconv_open(a(0), a(1))?,
            Gate::Iconv => self.iconv_run(a(0), [a(1), a(2), a(3), a(4)])?,
            Gate::IcClose => match self.iconv_descs.get_mut(a(0).wrapping_sub(1) as usize) {
                Some(d @ Some(_)) => {
                    *d = None;
                    0
                }
                _ => self.iconv_fail(EBADF)?,
            },
            _ => return Ok(None),
        }))
    }

    /// `iconv_open(to, from)`: a descriptor (1, 2, ...), or -1 with EINVAL.
    fn iconv_open(&mut self, to: u32, from: u32) -> Result<u32, Fault> {
        let name = |m: &Self, a| m.zstring(a).map(|s| charset::parse(&s));
        let (Some(to), Some(from)) = (name(self, to)?, name(self, from)?) else {
            return self.iconv_fail(EINVAL);
        };
        self.iconv_descs.push(Some((from, to)));
        Ok(self.iconv_descs.len() as u32)
    }

    /// `iconv(desc, **in, *inleft, **out, *outleft)`.
    fn iconv_run(&mut self, desc: u32, cells: [u32; 4]) -> Result<u32, Fault> {
        let Some(&Some((from, to))) = self.iconv_descs.get(desc.wrapping_sub(1) as usize) else {
            return self.iconv_fail(EBADF);
        };
        let [inp, inleft, outp, outleft] = cells;
        if inp == 0 || self.mem.read_u32(inp)? == 0 {
            return Ok(0); // reset: no state to reset
        }
        let (mut i, mut n) = (self.mem.read_u32(inp)?, self.mem.read_u32(inleft)?);
        let (mut o, mut room) = (self.mem.read_u32(outp)?, self.mem.read_u32(outleft)?);
        let mut errno = 0;
        while n > 0 {
            match self.iconv_one(from, to, i, n) {
                Ok((used, bytes)) if bytes.len() as u32 <= room => {
                    self.mem.write_bytes(o, &bytes)?;
                    (i, n, o, room) = (
                        i + used,
                        n - used,
                        o + bytes.len() as u32,
                        room - bytes.len() as u32,
                    );
                }
                Ok(_) => errno = E2BIG,
                Err(e) => errno = e,
            }
            if errno != 0 {
                break;
            }
        }
        for (cell, v) in [(inp, i), (inleft, n), (outp, o), (outleft, room)] {
            self.mem.write_u32(cell, v)?;
        }
        if errno != 0 {
            self.iconv_fail(errno)
        } else {
            Ok(0)
        }
    }

    /// One character from `from` at VM address `at` (`n` bytes left), as
    /// bytes in `to`; or the errno to stop with.
    fn iconv_one(&self, from: Enc, to: Enc, at: u32, n: u32) -> Result<(u32, Vec<u8>), u32> {
        let input = self.mem.slice(at, n.min(4)).map_err(|_| EINVAL)?;
        let (c, used) = charset::decode(from, input).map_err(|b| match b {
            Bad::Incomplete => EINVAL,
            Bad::Invalid => EILSEQ,
        })?;
        let bytes = charset::encode(to, c).ok_or(EILSEQ)?;
        Ok((used as u32, bytes))
    }

    /// Set errno and return C's `(size_t)-1`.
    fn iconv_fail(&mut self, errno: u32) -> Result<u32, Fault> {
        self.mem.write_u32(st::ERRNO, errno)?;
        Ok(FAIL)
    }

    fn zstring(&self, a: u32) -> Result<String, Fault> {
        let mut n = 0;
        while self.mem.read_u8(a + n)? != 0 {
            n += 1;
        }
        Ok(String::from_utf8_lossy(self.mem.slice(a, n)?).into_owned())
    }
}
