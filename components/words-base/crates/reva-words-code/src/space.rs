//! Code space housekeeping: `_align`, `here,`, `[`.

use crate::emit::Emit;
use reva_core::fault::Fault;
use reva_core::st;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Alignment, copying into code space, leaving compile state.
    pub trait Space: Emit {
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

        /// `[`: stop compiling.
        fn lbracket(&mut self) -> R {
            self.setv(st::DIDHEADER, 0)?;
            self.setv(st::IS_COMPILING, 0)
        }
    }
}
