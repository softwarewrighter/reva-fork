//! `_slurp` and `resize`: files and blocks in the heap.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::st;

type R<T = ()> = Result<T, Fault>;

reva_macros::machine_trait! {
    /// Heap words that move data.
    pub trait HeapWords {
        /// `PROC _slurp` ( a n -- a' n' ): read a whole file into the heap.
        /// Failure leaves ( 0 0 ), sets `ioerr` and CF.
        fn slurp(&mut self, host: &mut dyn Host) -> R<bool> {
            let n = self.eax();
            let a = self.nos()?;
            let path = String::from_utf8_lossy(self.mem.slice(a, n)?).into_owned();
            let bytes = host
                .read_file(&path)
                .or_else(|| host.fs().and_then(|fs| fs.read_whole(&path).ok()));
            let loaded = match bytes {
                Some(bytes) => match self.heap.alloc(bytes.len() as u32) {
                    Some(p) => {
                        self.mem.write_bytes(p, &bytes)?;
                        Some((p, bytes.len() as u32))
                    }
                    None => None,
                },
                None => None,
            };
            let (p, len) = loaded.unwrap_or((0, 0));
            self.set_nos(p)?;
            self.set_eax(len);
            self.cpu.cf = loaded.is_none();
            self.setv(st::IOR, loaded.is_none() as u32)?;
            Ok(loaded.is_some())
        }

        /// `resize` ( a n -- a' ): a new block with the old contents, or 0.
        fn resize(&mut self) -> R {
            let n = self.upop()?;
            let a = self.eax();
            let new = if a == 0 {
                self.heap.alloc(n)
            } else {
                match self.heap.size_of(a) {
                    Some(old) => {
                        let new = self.heap.alloc(n);
                        if let Some(p) = new {
                            self.mem.move_forward(a, p, old.min(n))?;
                            self.heap.free(a);
                        }
                        new
                    }
                    None => None,
                }
            };
            self.set_eax(new.unwrap_or(0));
            self.setv(st::IOR, new.is_none() as u32)
        }
    }
}
