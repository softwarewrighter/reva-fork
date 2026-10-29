//! DEFER vectors and list iteration: Forth called from Rust, nested.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Gate, st};

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Running Forth vectors and lists as nested calls.
    pub trait Vectors {
        /// Run a DEFER vector that takes no arguments, skipping it when it still
        /// points at `noop` (the common case, and the cheap one).
        fn call_vector(&mut self, host: &mut dyn Host, cell: u32) -> R {
            let t = self.var(cell);
            if t == Gate::Noop.addr() {
                return Ok(());
            }
            self.call_nested(host, t)
        }

        /// `type` ( a n -- ), through the vector if Forth redefined it.
        fn call_type(&mut self, host: &mut dyn Host) -> R {
            let t = self.var(st::TYPE);
            if t != Gate::OsType.addr() {
                return self.call_nested(host, t);
            }
            let n = self.upop()?;
            let a = self.upop()?;
            if n != 0 {
                host.write(self.mem.slice(a, n)?);
            }
            Ok(())
        }

        /// `emit` of `c`, through the vector if Forth redefined it.
        fn call_emit(&mut self, host: &mut dyn Host, c: u8) -> R {
            let t = self.var(st::EMIT);
            if t != Gate::OsEmit.addr() {
                self.upsh(c as u32)?;
                return self.call_nested(host, t);
            }
            host.write(&[c]);
            Ok(())
        }

        /// `PROC iterate`: call `xt` with each node's data address until it
        /// returns 0.  Runs as nested calls.
        fn iterate_list(&mut self, host: &mut dyn Host, xt: u32, list: u32) -> R {
            if xt == 0 {
                return Ok(());
            }
            let mut node = list;
            loop {
                node = self.mem.read_u32(node)?;
                if node == 0 {
                    return Ok(());
                }
                self.upsh(node + 4)?;
                self.call_nested(host, xt)?;
                if self.upop()? == 0 {
                    return Ok(());
                }
            }
        }
    }
}
