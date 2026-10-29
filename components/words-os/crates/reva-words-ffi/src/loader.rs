//! `(lib)`, `(-lib)` and `(func)`.

use crate::libs::{func_addr, lib_handle};
use reva_core::fault::{Fault, PORTABILITY_DOC};
use reva_core::host::Host;
use reva_core::x86::EAX;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Loading libraries and looking up functions.
    pub trait Loader {
        /// `PROC _loadlib` ( a n -- handle ).
        fn ffi_loadlib(&mut self) -> R {
            let n = self.ffi_pop()?;
            let a = self.reg(EAX);
            let name = self.ffi_string(a, n)?;
            let handle = lib_handle(&name);
            self.missing_lib = (handle == 0).then_some(name);
            self.set_reg(EAX, handle);
            Ok(())
        }

        /// `PROC _unloadlib` ( handle -- ): nothing to release.
        fn ffi_unloadlib(&mut self) -> R {
            self.ffi_pop().map(drop)
        }

        /// `PROC _osfunc` ( a n handle -- fn ): the address of a host function,
        /// or of a data symbol, or 0.  A lookup in a library `(lib)` could not
        /// serve (handle 0) notes that library once per session: `reva.f`
        /// resolves lazily, so the library's own load is silent until then.
        fn ffi_func(&mut self, host: &mut dyn Host) -> R {
            let handle = self.ffi_pop()?;
            let n = self.ffi_pop()?;
            let a = self.reg(EAX);
            let name = self.ffi_string(a, n)?;
            let addr = func_addr(handle, name.trim_end_matches('\0'));
            if handle == 0 {
                self.note_missing(host);
            }
            self.set_reg(EAX, addr);
            Ok(())
        }

        /// The note for a library `(lib)` could not serve, once per session.
        fn note_missing(&mut self, host: &mut dyn Host) {
            if let Some(lib) = self.missing_lib.take()
                && !self.reported_libs.contains(&lib)
            {
                host.diag(&format!(
                    "note: native library {lib} is not available in the portable core \
                     (see {PORTABILITY_DOC})\n"
                ));
                self.reported_libs.push(lib);
            }
        }
    }
}
