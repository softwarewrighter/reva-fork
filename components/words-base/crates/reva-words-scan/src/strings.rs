//! Strings and numbers in VM memory: lengths, `compare`, `(.r)`,
//! `single`, `double`.

use reva_core::fault::Fault;
use reva_core::layout::NPAD;
use reva_core::st;
use reva_words_text::compare::{first_difference, length_order};
use reva_words_text::number::{format_r, parse_double, parse_single};

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// String and number words over VM memory.
    pub trait Strings {
        /// The length of the zero-terminated string at `a`.
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
            let r = if n1 == 0 || a1 == 0 || n2 == 0 || a2 == 0 {
                length_order(n1, n2)
            } else {
                let len = (n1 as i32).min(n2 as i32) as u32;
                let (s1, s2) = (self.mem.slice(a1, len)?, self.mem.slice(a2, len)?);
                first_difference(s1, s2, fold).unwrap_or(n1.wrapping_sub(n2) as i32)
            };
            self.set_eax(r as u32);
            Ok(())
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
            let text = format_r(n, base, width, self.var(st::PADCHAR) as u8);
            let p = NPAD - text.len() as u32;
            self.mem.write_bytes(p, &text)?;
            self.set_eax(p);
            self.upsh(text.len() as u32)
        }

        /// `single` of the string `a n` in the current base.
        fn single_at(&self, a: u32, n: u32) -> R<Option<u32>> {
            if n == 0 {
                return Ok(None);
            }
            Ok(parse_single(self.mem.slice(a, n)?, self.var(st::BASE)))
        }

        /// `double` of the string `a n` in the current base.
        fn double_at(&self, a: u32, n: u32) -> R<Option<(u32, u32)>> {
            if n < 2 {
                return Ok(None);
            }
            Ok(parse_double(self.mem.slice(a, n)?, self.var(st::BASE)))
        }
    }
}
