//! Ending definitions: `;;` (with its tail call) and `;inline`.

use crate::emit::Emit;
use crate::space::Space;
use reva_core::Gate;
use reva_core::fault::Fault;
use reva_core::layout::DTOP;
use reva_core::st;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// `;;` and `;inline`.
    pub trait Semi: Emit, Space {
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
            self.mem.write_u32(entry.wrapping_sub(4), Gate::IClass.addr())?;
            self.mem.write_u32(entry + 4, xt)?;
            self.lbracket()
        }
    }
}
