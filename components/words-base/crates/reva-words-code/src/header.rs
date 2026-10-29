//! `__header`: dictionary entries.

use crate::space::Space;
use reva_core::Gate;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::layout::DTOP;
use reva_core::st;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Making a dictionary entry.
    pub trait Header: Space {
        /// `PROC __header` ( a n -- ): create a dictionary entry
        /// `[class][link][xt][count][name...]` for the word about to be compiled
        /// at `here`, remembering enough to undo it if compilation fails.
        fn header_default(&mut self, host: &mut dyn Host) -> R {
            let (d, lastp, prev) = self.header_start()?;
            let n = self.upop()?;
            let a = self.upop()?;
            if d.checked_add(13 + n).is_none_or(|e| e > DTOP) {
                return self.dictionary_full(host);
            }
            self.write_entry(d, lastp, prev, (a, n))
        }

        /// Align, and remember `here`, the entry's place and the old link
        /// (for rolling back): ( entry place, `last` cell, previous entry ).
        fn header_start(&mut self) -> R<(u32, u32, u32)> {
            self.setv(st::DIDHEADER, 1)?;
            self.align_code()?;
            let d = (self.var(st::D) + 3) & !3;
            self.setv(st::LASTH, self.here())?;
            self.setv(st::LASTD, d)?;
            let lastp = self.var(st::LAST);
            let prev = self.mem.read_u32(lastp)?;
            self.setv(st::LASTL, prev)?;
            Ok((d, lastp, prev))
        }

        /// No room for the entry: run `dictgone` if Forth redefined it.
        fn dictionary_full(&mut self, host: &mut dyn Host) -> R {
            let v = self.var(st::DICTGONE);
            if v != Gate::OsBye.addr() {
                return self.call_nested(host, v);
            }
            Err(Fault::OutOfSpace("dictionary"))
        }

        /// Write the entry at `d` and link it in front of `prev`.
        fn write_entry(&mut self, d: u32, lastp: u32, prev: u32, name: (u32, u32)) -> R {
            let (a, n) = name;
            let entry = d + 4;
            self.mem.write_u32(d, self.var(st::DEFAULT_CLASS))?;
            self.mem.write_u32(lastp, entry)?;
            self.mem.write_u32(entry, prev)?;
            self.mem.write_u32(entry + 4, self.here())?;
            self.mem.write_u8(entry + 8, n as u8)?;
            self.mem.move_forward(a, entry + 9, n)?;
            self.setv(st::D, entry + 9 + n)
        }
    }
}
