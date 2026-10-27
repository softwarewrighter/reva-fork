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

use crate::heap::Heap;
use crate::layout::*;
use crate::machine::{Machine, st};

const MAGIC: &[u8; 8] = b"REVAVX86";
const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageError {
    BadMagic,
    BadVersion(u32),
    Truncated,
    BadSegment { start: u32, len: u32 },
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageError::BadMagic => write!(f, "not a Reva vx86 image"),
            ImageError::BadVersion(v) => write!(f, "unsupported image version {v}"),
            ImageError::Truncated => write!(f, "image is truncated"),
            ImageError::BadSegment { start, len } => {
                write!(f, "image segment {start:#x}+{len:#x} is out of range")
            }
        }
    }
}

impl std::error::Error for ImageError {}

impl Machine {
    /// Serialize the machine's persistent state: between [`Machine::run`]
    /// calls, or from `(save)` while Forth code runs (loading resets the
    /// interpreter's source and catch state, so either works).
    pub fn save_image(&self) -> Vec<u8> {
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
    pub fn load_image(bytes: &[u8], args: &[&str]) -> Result<Machine, ImageError> {
        let mut r = Reader { b: bytes, at: 0 };
        if r.take(8)? != MAGIC {
            return Err(ImageError::BadMagic);
        }
        let version = r.u32()?;
        if version != VERSION {
            return Err(ImageError::BadVersion(version));
        }
        let mem_size = r.u32()?;
        let mut m = Machine::with_memory(mem_size);
        for _ in 0..r.u32()? {
            let (start, len) = (r.u32()?, r.u32()?);
            let data = r.take(len as usize)?;
            m.mem
                .write_bytes(start, data)
                .map_err(|_| ImageError::BadSegment { start, len })?;
        }
        let mut words = std::iter::from_fn(|| r.u32().ok());
        m.heap = Heap::load(&mut words).ok_or(ImageError::Truncated)?;
        // A saved program started with no source file and no catch frame,
        // whatever was running when `(save)` wrote the image.
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
        m.set_args(args);
        m.reset_cpu();
        Ok(m)
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ImageError> {
        let end = self.at.checked_add(n).ok_or(ImageError::Truncated)?;
        let s = self.b.get(self.at..end).ok_or(ImageError::Truncated)?;
        self.at = end;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, ImageError> {
        let s = self.take(4)?;
        Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
    }
}
