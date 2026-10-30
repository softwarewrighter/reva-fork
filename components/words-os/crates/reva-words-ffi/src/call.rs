//! `(call)`: a cdecl call of a virtual library's function.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::{EAX, ESI};
use reva_core::{Gate, Next};
use reva_core_ffi::args::CallArgs;
use reva_core_ffi::regs::MachineRegs;
use reva_vlib::highlight::Highlight;
use reva_vlib::iconv::Iconv;
use reva_vlib::sqlite::SqliteLib;
use reva_vlib::turtle::Turtle;
use reva_words_files::dirs::Dirs;
use reva_words_libc::env::Env;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Calling host functions.
    pub trait Call: Dirs, Env, Highlight, Iconv, SqliteLib, Turtle {
        /// `PROC _call` ( x1 .. xn fn n -- result ): a cdecl call.  Only host
        /// functions can be called; the first C argument is the deepest of the
        /// `n` cells, as when the original pushed them onto the C stack.
        fn ffi_call(&mut self, host: &mut dyn Host) -> R<Next> {
            // Host calls read their arguments in place and pop them only once
            // the answer is in, so a call that waits can be retried.
            let n = self.reg(EAX);
            let f = self.mem.read_u32(self.reg(ESI))?;
            if let Some(next) = Gate::from_addr(f).and_then(|g| self.sqlite_call(host, g, n)) {
                return next;
            }
            let (f, args) = self.call_args()?;
            let g = Gate::from_addr(f).ok_or(Fault::NativeCall { addr: f })?;
            let result = match self.libc_call(host, g, &args)? {
                Some(r) => r,
                None => self.vlib_call(host, g, &args)?.ok_or(Fault::NativeCall { addr: f })?,
            };
            self.ffi_push(result)?;
            Ok(Next::Ret)
        }

        /// Pop `n`, the function, and its `n` arguments (first C argument first).
        fn call_args(&mut self) -> R<(u32, Vec<u32>)> {
            let n = self.ffi_pop()?;
            let f = self.ffi_pop()?;
            if n > 16 {
                return Err(Fault::NotImplemented("(call) with more than 16 parameters"));
            }
            let mut args = vec![0u32; n as usize];
            for i in (0..n as usize).rev() {
                args[i] = self.ffi_pop()?;
            }
            Ok((f, args))
        }

        /// A function of the virtual libc, or `None`.
        fn libc_call(&mut self, host: &mut dyn Host, g: Gate, args: &[u32]) -> R<Option<u32>> {
            let arg0 = args.first().copied().unwrap_or(0);
            Ok(Some(match g {
                Gate::HostGetenv => self.c_getenv(host, arg0)?,
                Gate::HostPutenv => self.c_putenv(host, arg0)?,
                Gate::HostLocaltime => self.c_localtime(arg0)?,
                Gate::HostOpendir => self.c_opendir(host, arg0)?,
                Gate::HostReaddir => self.c_readdir(host, arg0)?,
                Gate::HostClosedir => self.c_closedir(arg0)?,
                Gate::HostSystem => self.c_system(host, arg0)?,
                _ => return Ok(None),
            }))
        }

        /// A function of reva-vlib's libraries (iconv, highlight, turtle), or
        /// `None`.
        fn vlib_call(&mut self, host: &mut dyn Host, g: Gate, args: &[u32]) -> R<Option<u32>> {
            if let Some(r) = self.iconv_call(g, args)? {
                return Ok(Some(r));
            }
            if let Some(r) = self.highlight_call(host, g, args)? {
                return Ok(Some(r));
            }
            self.turtle_call(host, g, args)
        }
    }
}
