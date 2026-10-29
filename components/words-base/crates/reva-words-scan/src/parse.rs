//! Parsing the input buffer: `parsews`, `parse`, `parse/`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::{PARSE_BUF, PARSE_BUF_SIZE};
use reva_core::st;
use reva_words_calls::vectors::Vectors;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Words out of the input buffer (`tib`/`>in`, or an `eval` string).
    pub trait Parse: Vectors {
        /// `PROC parsews`: skip blanks (9..=32), return the next word.  Each
        /// newline crossed runs the `do_cr` vector (line counting in `include`).
        fn parsews(&mut self, host: &mut dyn Host) -> R<(u32, u32)> {
            let end = self.var(st::TP);
            let start = self.skip_blanks(host, end)?;
            let mut p = start;
            while p < end && !(9..=32).contains(&self.mem.read_u8(p)?) {
                p += 1;
            }
            self.step_past(host, p, end)?;
            Ok((start, p - start))
        }

        /// Skip blanks from `>in`; newlines advance `>in` and run `do_cr`.
        fn skip_blanks(&mut self, host: &mut dyn Host, end: u32) -> R<u32> {
            let mut p = self.var(st::TIN);
            while p < end {
                let c = self.mem.read_u8(p)?;
                if c == b'\n' {
                    p += 1;
                    self.setv(st::TIN, p)?;
                    self.call_vector(host, st::DO_CR)?;
                    continue;
                }
                if !(9..=32).contains(&c) {
                    break;
                }
                p += 1;
            }
            Ok(p)
        }

        /// Set `>in` past the blank that ended a word (running `do_cr` if
        /// it was a newline), or to the end.
        fn step_past(&mut self, host: &mut dyn Host, p: u32, end: u32) -> R {
            if p >= end {
                return self.setv(st::TIN, p);
            }
            let nl = self.mem.read_u8(p)? == b'\n';
            self.setv(st::TIN, p + 1)?;
            if nl {
                self.call_vector(host, st::DO_CR)?;
            }
            Ok(())
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
    }
}
