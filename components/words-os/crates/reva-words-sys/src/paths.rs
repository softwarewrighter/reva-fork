//! Paths in VM memory, and `ioerr`.

use reva_core::fault::Fault;
use reva_core::st;
use reva_core::vfs::{Errno, errno::EFAULT};

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Paths out of VM memory; `ioerr` after a call.
    pub trait Paths {
        /// The path `a n`.
        fn path_at(&self, a: u32, n: u32) -> R<String> {
            Ok(String::from_utf8_lossy(self.mem.slice(a, n)?).into_owned())
        }

        /// A zero-terminated path, or `EFAULT` if it runs off memory.
        fn zpath(&self, a: u32) -> Result<String, Errno> {
            let mut n = 0;
            loop {
                match self.mem.read_u8(a.wrapping_add(n)) {
                    Ok(0) => break,
                    Ok(_) => n += 1,
                    Err(_) => return Err(EFAULT),
                }
            }
            self.path_at(a, n).map_err(|_| EFAULT)
        }

        /// `ioerr` after a call: 0, or the negative result.
        fn set_ior(&mut self, result: u32) -> R {
            let v = if (result as i32) < 0 { result } else { 0 };
            self.setv(st::IOR, v)
        }
    }
}
