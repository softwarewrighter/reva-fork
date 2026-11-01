//! Bytes into code space: `compile`, `literal`, `,`.

use reva_core::fault::Fault;
use reva_core::layout::HTOP;
use reva_core::st;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Code generation at `here`.
    pub trait Emit {
        /// `here`, if `n` more bytes of code fit.
        fn reserve_code(&self, n: u32) -> R<u32> {
            let h = self.here();
            if h.checked_add(n).is_none_or(|e| e > HTOP) {
                return Err(Fault::OutOfSpace("code"));
            }
            Ok(h)
        }

        /// Append `bytes` at `here`.
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

        /// `PROC comma` and friends: store TOS at `here`, advance by `n`.
        /// Like the original, all four bytes are written whatever `n` is.
        fn comma_n(&mut self, n: u32) -> R {
            let h = self.reserve_code(4)?;
            self.mem.write_u32(h, self.eax())?;
            self.setv(st::H, h + n)?;
            self.drop1()
        }
    }
}
