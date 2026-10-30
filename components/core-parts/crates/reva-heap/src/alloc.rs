//! Allocating and freeing.

use crate::heap::{ALIGN, Heap};

impl Heap {
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
}
