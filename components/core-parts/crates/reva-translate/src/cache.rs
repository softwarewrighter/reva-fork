//! The decode cache: instructions by address, with a fast hasher.

use reva_x86::Insn;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// Fast hasher for `u32` code addresses.
#[derive(Default)]
pub struct AddrHasher(u64);

impl Hasher for AddrHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ b as u64).wrapping_mul(0x100_0000_01b3);
        }
    }
    fn write_u32(&mut self, n: u32) {
        self.0 = (n as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Decoded instructions by address.
pub type CodeCache = HashMap<u32, Insn, BuildHasherDefault<AddrHasher>>;
