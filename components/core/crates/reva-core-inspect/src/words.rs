//! The dictionary: lookup, naming addresses, rollback.

use reva_core::fault::Fault;
use reva_core::{Gate, st};

reva_macros::machine_trait! {
    /// The dictionary as data.
    pub trait Dictionary {
        /// Dictionary search (`__find`), returning the entry address.
        fn find_entry(&self, name: &[u8]) -> Option<u32> {
            let mut e = self.mem.read_u32(self.var(st::LAST)).ok()?;
            while e != 0 {
                let n = self.mem.read_u8(e + 8).ok()?;
                if n as usize == name.len() & 0xff
                    && let Ok(s) = self.mem.slice(e + 9, name.len() as u32)
                    && s == name
                {
                    return Some(e);
                }
                e = self.mem.read_u32(e).ok()?;
            }
            None
        }

        /// Look a word up in the current search order; returns its xt.
        fn find(&self, name: &str) -> Option<u32> {
            self.find_entry(name.as_bytes())
                .and_then(|e| self.mem.read_u32(e + 4).ok())
        }

        /// Undo a half-compiled definition (`interpret`'s `.fail` path).
        fn rollback_definition(&mut self) -> Result<(), Fault> {
            if self.var(st::IS_COMPILING) != 0 && self.var(st::DIDHEADER) != 0 {
                let lasth = self.var(st::LASTH);
                let lastd = self.var(st::LASTD);
                let lastl = self.var(st::LASTL);
                let lastp = self.var(st::LAST);
                self.mem.write_u32(st::H, lasth)?;
                self.mem.write_u32(st::D, lastd)?;
                self.mem.write_u32(lastp, lastl)?;
            }
            self.mem.write_u32(st::IS_COMPILING, 0)?;
            self.mem.write_u32(st::DIDHEADER, 0)
        }

        /// Name the code at `addr`: a core routine, or the dictionary word whose
        /// code starts closest below it (with the offset into it).  Used in
        /// fault reports so a crash points at Forth source, not just a number.
        fn describe_addr(&self, addr: u32) -> String {
            if let Some(g) = Gate::from_addr(addr) {
                return format!("core:{}", g.name());
            }
            match self.nearest_word(addr) {
                Some((xt, e)) => {
                    let n = self.mem.read_u8(e + 8).unwrap_or(0) as u32;
                    let name = self.mem.slice(e + 9, n).unwrap_or(b"?");
                    format!("{}+{:#x}", String::from_utf8_lossy(name), addr - xt)
                }
                None => format!("{addr:#010x}"),
            }
        }

        /// The word whose code starts closest at or below `addr`: (xt, entry).
        fn nearest_word(&self, addr: u32) -> Option<(u32, u32)> {
            let mut best: Option<(u32, u32)> = None;
            let mut e = self.mem.read_u32(self.var(st::LAST)).unwrap_or(0);
            let mut guard = 0;
            while e != 0 && guard < 1 << 20 {
                let xt = self.mem.read_u32(e + 4).unwrap_or(0);
                if xt <= addr && best.is_none_or(|(bx, _)| xt > bx) {
                    best = Some((xt, e));
                }
                e = self.mem.read_u32(e).unwrap_or(0);
                guard += 1;
            }
            best
        }
    }
}
