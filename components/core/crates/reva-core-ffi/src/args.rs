//! Strings and cells of a `(call)`.

use crate::regs::MachineRegs;
use reva_core::fault::Fault;
use reva_core::x86::{EAX, ESI};

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// `(call)`'s arguments.
    pub trait CallArgs: MachineRegs {
        /// The string `a n`.
        fn ffi_string(&self, a: u32, n: u32) -> R<String> {
            Ok(String::from_utf8_lossy(self.mem.slice(a, n)?).into_owned())
        }

        /// The zero-terminated string at `a`.
        fn ffi_zstring(&self, a: u32) -> R<String> {
            let mut n = 0;
            while self.mem.read_u8(a + n)? != 0 {
                n += 1;
            }
            self.ffi_string(a, n)
        }

        /// Pop the top data-stack cell (EAX) — the `upop` macro.
        fn ffi_pop(&mut self) -> R<u32> {
            let v = self.reg(EAX);
            let sp = self.reg(ESI);
            self.set_reg(EAX, self.mem.read_u32(sp)?);
            self.set_reg(ESI, sp.wrapping_add(4));
            Ok(v)
        }

        /// upsh: `v` becomes TOS.
        fn ffi_push(&mut self, v: u32) -> R {
            let sp = self.reg(ESI).wrapping_sub(4);
            self.mem.write_u32(sp, self.reg(EAX))?;
            self.set_reg(ESI, sp);
            self.set_reg(EAX, v);
            Ok(())
        }

        /// End a `(call)` of `n` cells read in place: drop them, the function
        /// address and the count, and push `result`.
        fn ffi_finish(&mut self, n: u32, result: u32) -> R {
            for _ in 0..n + 2 {
                self.ffi_pop()?;
            }
            self.ffi_push(result)
        }
    }
}
