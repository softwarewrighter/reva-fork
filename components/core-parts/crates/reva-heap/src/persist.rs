//! The allocator's state in an image.

use crate::heap::Heap;

impl Heap {
    /// Append the allocator state to `out`.
    pub fn save(&self, out: &mut Vec<u8>) {
        for v in [
            self.base,
            self.limit,
            self.top,
            self.used.len() as u32,
            self.free.len() as u32,
        ] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for (a, n) in self.used.iter().chain(self.free.iter()) {
            out.extend_from_slice(&a.to_le_bytes());
            out.extend_from_slice(&n.to_le_bytes());
        }
    }

    /// The allocator state written by [`Heap::save`].
    pub fn load(words: &mut impl Iterator<Item = u32>) -> Option<Heap> {
        let (base, limit, top, nu, nf) = (
            words.next()?,
            words.next()?,
            words.next()?,
            words.next()?,
            words.next()?,
        );
        let mut h = Heap::new(base, limit);
        h.top = top;
        for _ in 0..nu {
            h.used.insert(words.next()?, words.next()?);
        }
        for _ in 0..nf {
            h.free.insert(words.next()?, words.next()?);
        }
        Some(h)
    }
}
