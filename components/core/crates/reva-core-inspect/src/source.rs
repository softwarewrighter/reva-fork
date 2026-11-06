//! Where the interpreter is: the file and line being included (with the
//! files that include it), the word just read, and the stack, for error,
//! fault and warning messages (CoSy's ask 15).

use reva_core::layout::{RP0, S0};
use reva_core::x86::{EAX, ESI, ESP};
use reva_core::{Gate, st};

use crate::words::Dictionary;

/// The most callers a fault report names.
pub const CALLERS: usize = 8;

reva_macros::machine_trait! {
    /// The interpreter's position, for messages.
    pub trait Source {
        /// `FILE:LINE`, then ` (from FILE:LINE, ...)` for the files that
        /// include it; `None` when the input is not a file being included.
        fn source_where(&self) -> Option<String> {
            let mut pos = self.var(st::TIN);
            let inside = |f: &reva_core::IncludeFrame, p: u32| (f.start..=f.end).contains(&p);
            let mut k = self.includes.iter().rposition(|f| inside(f, pos))?;
            let mut at = Vec::new();
            loop {
                let f = &self.includes[k];
                let text = self.mem.slice(f.start, pos - f.start).ok()?;
                // reading a word at the end of a line takes its newline too
                let text = text.strip_suffix(b"\n").unwrap_or(text);
                let line = 1 + text.iter().filter(|c| **c == b'\n').count();
                at.push(format!("{}:{line}", f.name));
                pos = f.from;
                match self.includes[..k].iter().rposition(|g| inside(g, pos)) {
                    Some(j) => k = j,
                    None => break,
                }
            }
            let (first, outer) = at.split_first()?;
            Some(match outer.is_empty() {
                true => first.clone(),
                false => format!("{first} (from {})", outer.join(", from ")),
            })
        }

        /// The word just read from the input (the one before `TIN`).
        fn last_word(&self) -> Option<String> {
            let end = self.var(st::TIN);
            let start = end.saturating_sub(64);
            let text = self.mem.slice(start, end - start).ok()?;
            // a word is printable characters (the input buffer may hold NULs)
            let end = text.iter().rposition(|c| c.is_ascii_graphic() || *c >= 0x80)? + 1;
            let text = &text[..end];
            let word = text.rsplit(|c| !(c.is_ascii_graphic() || *c >= 0x80)).next()?;
            (!word.is_empty()).then(|| String::from_utf8_lossy(word).into_owned())
        }

        /// ` at FILE:LINE after WORD` (or just ` after WORD`), for a message.
        fn where_suffix(&self) -> String {
            let at = self.source_where().map(|w| format!(" at {w}")).unwrap_or_default();
            let after = self.last_word().map(|w| format!(" after `{w}`")).unwrap_or_default();
            format!("{at}{after}")
        }

        /// The words that called the code running now, innermost first, by
        /// name (CoSy's ask 22): the return stack's cells that point just
        /// after a `call rel32` in a dictionary word, up to the first core
        /// routine's address (the outer interpreter running a word, a nested
        /// call's return), at most [`CALLERS`]. Cells `>r` put there rarely
        /// look like that; a tail call (a jump) leaves no cell, so its word is
        /// not shown.
        fn callers(&self) -> Vec<String> {
            let (esp, here) = (self.cpu.r[ESP as usize], self.var(st::H));
            let call_at = |v: u32| v > 5 && v <= here && self.mem.read_u8(v - 5).ok() == Some(0xe8);
            (esp..RP0)
                .step_by(4)
                .map_while(|a| self.mem.read_u32(a).ok())
                .take_while(|v| Gate::from_addr(*v).is_none())
                .filter(|v| call_at(*v))
                .filter_map(|v| self.nearest_word(v - 5))
                .map(|(_, e)| {
                    let n = self.mem.read_u8(e + 8).unwrap_or(0) as u32;
                    String::from_utf8_lossy(self.mem.slice(e + 9, n).unwrap_or(b"?")).into_owned()
                })
                .take(CALLERS)
                .collect()
        }

        /// The data stack as `.s` shows it: depth, then up to 8 cells, top
        /// last; `None` below empty.
        fn stack_line(&self) -> Option<String> {
            let esi = self.cpu.r[ESI as usize];
            let depth = (S0 as i64 - esi as i64) / 4;
            if !(0..=4096).contains(&depth) {
                return None;
            }
            // the top is in EAX, the one below it at [ESI], then upwards
            let shown = depth.min(8) as u32;
            let below = (2..=shown).rev().filter_map(|i| self.mem.read_u32(esi + 4 * (i - 2)).ok());
            let top = (depth > 0).then_some(self.cpu.r[EAX as usize]);
            let cells: Vec<String> = below.chain(top).map(|v| (v as i32).to_string()).collect();
            Some(match cells.is_empty() {
                true => format!("stack ({depth})"),
                false => format!("stack ({depth}): {}", cells.join(" ")),
            })
        }
    }
}
