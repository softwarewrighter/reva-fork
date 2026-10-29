//! `query`'s line editor, one key at a time.

use reva_core::fault::Fault;
use reva_core::layout::{EOF_KEY, TIB, TIB_TOP};
use reva_core::{Gate, Next, st};

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Keys into the terminal input buffer.
    pub trait Query {
        /// One key of `query`'s line editor.  `None` means "keep reading".
        fn query_char(&mut self, c: u32) -> R<Option<Next>> {
            let tp = self.var(st::TP);
            if c == EOF_KEY {
                return self.query_eof(tp).map(Some);
            }
            if c == u32::MAX {
                return Ok(Some(Next::Jump(Gate::Interpret.addr())));
            }
            self.edit_char(c as u8, tp)
        }

        /// End of input.  Deviation: newcore exited at once, losing a final
        /// line that had no newline.  Interpret that line first; the next
        /// EOF exits.
        fn query_eof(&mut self, tp: u32) -> R<Next> {
            if tp > TIB {
                return Ok(Next::Jump(Gate::InterpWord.addr()));
            }
            self.upsh(0)?;
            Ok(Next::Jump(Gate::Bye.addr()))
        }

        /// A key into the line: newline ends it, backspace edits, a full
        /// buffer ends it too.
        fn edit_char(&mut self, ch: u8, tp: u32) -> R<Option<Next>> {
            match ch {
                b'\n' => return Ok(Some(Next::Jump(Gate::InterpWord.addr()))),
                b'\r' => {}
                8 => self.setv(st::TP, if tp > TIB { tp - 1 } else { TIB })?,
                _ => {
                    self.mem.write_u8(tp, if ch == b'\t' { b' ' } else { ch })?;
                    self.setv(st::TP, tp + 1)?;
                    if tp + 1 >= TIB_TOP {
                        return Ok(Some(Next::Jump(Gate::InterpWord.addr())));
                    }
                }
            }
            Ok(None)
        }
    }
}
