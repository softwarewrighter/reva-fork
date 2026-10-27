//! A small first-fit allocator for `allocate`/`free`/`resize` and `slurp`.
//!
//! Book-keeping lives on the Rust side (not in VM memory) so a wild store
//! from Forth cannot corrupt the allocator; [`Heap::save`]/[`Heap::load`]
//! carry it through images.

use std::collections::BTreeMap;

pub struct Heap {
    base: u32,
    limit: u32,
    top: u32,
    used: BTreeMap<u32, u32>,
    free: BTreeMap<u32, u32>,
}

const ALIGN: u32 = 8;

impl Heap {
    pub fn new(base: u32, limit: u32) -> Heap {
        Heap {
            base,
            limit,
            top: base,
            used: BTreeMap::new(),
            free: BTreeMap::new(),
        }
    }

    /// Highest address handed out so far (end of the used heap).
    pub fn top(&self) -> u32 {
        self.top
    }

    pub fn size_of(&self, addr: u32) -> Option<u32> {
        self.used.get(&addr).copied()
    }

    /// Allocate `n` bytes; `None` when the heap is exhausted.
    pub fn alloc(&mut self, n: u32) -> Option<u32> {
        let n = n.max(1).checked_add(ALIGN - 1)? & !(ALIGN - 1);
        let hit = self
            .free
            .iter()
            .find(|&(_, &sz)| sz >= n)
            .map(|(&a, &sz)| (a, sz));
        let addr = match hit {
            Some((a, sz)) => {
                self.free.remove(&a);
                if sz > n {
                    self.free.insert(a + n, sz - n);
                }
                a
            }
            None => {
                let a = self.top;
                let end = a.checked_add(n)?;
                if end > self.limit {
                    return None;
                }
                self.top = end;
                a
            }
        };
        self.used.insert(addr, n);
        Some(addr)
    }

    /// Free a block.  Returns false for an address that was not allocated
    /// (Reva's C `free` would have crashed; we just report it).
    pub fn free(&mut self, addr: u32) -> bool {
        let Some(mut n) = self.used.remove(&addr) else {
            return false;
        };
        let mut a = addr;
        if let Some((&pa, &pn)) = self.free.range(..a).next_back()
            && pa + pn == a
        {
            self.free.remove(&pa);
            a = pa;
            n += pn;
        }
        if let Some(&nn) = self.free.get(&(a + n)) {
            self.free.remove(&(a + n));
            n += nn;
        }
        if a + n == self.top {
            self.top = a;
        } else {
            self.free.insert(a, n);
        }
        true
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_free_reuse_and_coalesce() {
        let mut h = Heap::new(0x1000, 0x2000);
        let a = h.alloc(10).unwrap();
        let b = h.alloc(10).unwrap();
        let c = h.alloc(10).unwrap();
        assert_eq!((a, b, c), (0x1000, 0x1010, 0x1020));
        assert!(h.free(a));
        assert!(h.free(b));
        assert_eq!(h.alloc(32).unwrap(), 0x1000); // a+b coalesced
        assert!(h.free(c));
        assert!(!h.free(c));
        assert!(h.alloc(0x2000).is_none());
    }
}
