//! # Images
//!
//! An image is the state Reva's `(save)` wrote into an executable: statics,
//! dictionary, code, and heap — here without the executable around it, so
//! the same image runs on every host.  It holds vx86 bytes only; decode
//! caches and native translations are rebuilt after loading.
//!
//! Format (little-endian):
//!
//! ```text
//! magic "REVAVX86"  u32 version  u32 mem_size
//! u32 segment count, then per segment: u32 start, u32 len, bytes
//! heap allocator state (see heap.rs)
//! ```
//!
//! Loading starts the machine at `cold`, as a saved Reva program did.

use crate::boot::Boot;
pub use crate::format::ImageError;
use crate::format::Reader;
use reva_core::heap::Heap;
use reva_core::layout::*;
use reva_core::{Machine, st};

const MAGIC: &[u8; 8] = b"REVAVX86";
const VERSION: u32 = 1;

reva_macros::machine_trait! {
    /// Saving and loading images.
    pub trait Images: Boot {
        /// Serialize the machine's persistent state: between [`Machine::run`]
        /// calls, or from `(save)` while Forth code runs (loading resets the
        /// interpreter's source and catch state, so either works).
        fn save_image(&self) -> Vec<u8> {
            let mut out = Vec::new();
            out.extend_from_slice(MAGIC);
            out.extend_from_slice(&VERSION.to_le_bytes());
            out.extend_from_slice(&self.mem.size().to_le_bytes());
            let segments = [
                (GATE_END, D0),
                (D0, self.var(st::D)),
                (H0, self.var(st::H)),
                (HEAP_BASE, self.heap.top()),
            ];
            out.extend_from_slice(&(segments.len() as u32).to_le_bytes());
            for (start, end) in segments {
                let len = end.saturating_sub(start);
                out.extend_from_slice(&start.to_le_bytes());
                out.extend_from_slice(&len.to_le_bytes());
                out.extend_from_slice(self.mem.slice(start, len).unwrap_or(&[]));
            }
            self.heap.save(&mut out);
            out
        }

        /// Recreate a machine from an image; `args` replace `argc`/`(argv)`.
        fn load_image(bytes: &[u8], args: &[&str]) -> Result<Machine, ImageError> {
            let mut r = Reader { b: bytes, at: 0 };
            if r.take(8)? != MAGIC {
                return Err(ImageError::BadMagic);
            }
            let version = r.u32()?;
            if version != VERSION {
                return Err(ImageError::BadVersion(version));
            }
            let mut m = Machine::with_memory(r.u32()?);
            read_segments(&mut r, &mut m)?;
            let mut words = std::iter::from_fn(|| r.u32().ok());
            m.heap = Heap::load(&mut words).ok_or(ImageError::Truncated)?;
            start_state(&mut m)?;
            m.set_args(args);
            m.reset_cpu();
            Ok(m)
        }
    }
}

/// The image's memory segments, into `m`.
fn read_segments(r: &mut Reader, m: &mut Machine) -> Result<(), ImageError> {
    for _ in 0..r.u32()? {
        let (start, len) = (r.u32()?, r.u32()?);
        let data = r.take(len as usize)?;
        m.mem
            .write_bytes(start, data)
            .map_err(|_| ImageError::BadSegment { start, len })?;
    }
    Ok(())
}

/// A saved program started with no source file and no catch frame,
/// whatever was running when `(save)` wrote the image.
fn start_state(m: &mut Machine) -> Result<(), ImageError> {
    for (var, v) in [
        (st::SOURCE, 0),
        (st::TP, TIB),
        (st::TIN, TIB),
        (st::IS_COMPILING, 0),
        (st::HANDLER, 0),
    ] {
        m.mem
            .write_u32(var, v)
            .map_err(|_| ImageError::BadSegment { start: var, len: 4 })?;
    }
    Ok(())
}
