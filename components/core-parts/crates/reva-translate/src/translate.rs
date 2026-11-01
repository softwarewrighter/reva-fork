//! # Hooks for native and WebAssembly translators
//!
//! The vx86 interpreter is the reference tier: it runs everywhere and defines
//! the semantics.  Faster tiers translate *basic blocks* of vx86 code into
//! host code and keep the result as a cache:
//!
//! * **x86-64**: nearly 1:1.  VM registers map to host 32-bit registers,
//!   memory operands gain a base register (`[r15 + rax]`), ESP becomes a
//!   pinned register holding the VM return-stack offset.
//! * **ARM64**: one or two instructions per vx86 instruction; flags map to
//!   NZCV (note: AArch64's carry after subtraction is inverted relative to
//!   x86's borrow), memory is `[x28, w_n, uxtw]`.
//! * **WebAssembly**: a block becomes a function over `(memory, regs)`; the
//!   browser's JIT does the rest.
//!
//! Gateway addresses (Rust core routines) always end a block and are called
//! through the interpreter's dispatcher, so translators never need to know
//! about them beyond "exit to the dispatcher with EIP = gateway".
//!
//! The contract with the rest of the VM is small:
//!
//! 1. A translator sees a [`Block`] produced by [`decode_block`]; it may
//!    decline (`None`) and the interpreter keeps running that block.
//! 2. Translations are invalidated by page when code is written; the machine
//!    forwards every dirty code page to [`Translator::invalidate`].
//! 3. Images never contain translations — only vx86 bytes — so an image runs
//!    on every host and translations are rebuilt lazily after loading.
//!
//! Step 1 provides the block decoder, the invalidation plumbing, and this
//! trait.  No native translator is installed yet.

use reva_x86::fault::Fault;
use reva_x86::layout::{GATE_BASE, GATE_END};
use reva_x86::mem::Memory;
use reva_x86::{Insn, decode};

/// Straight-line vx86 code: `insns` run in order; the last one (if
/// `ends_with_branch`) transfers control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub start: u32,
    pub insns: Vec<(u32, Insn)>,
    /// First address after the block.
    pub end: u32,
    pub ends_with_branch: bool,
}

/// Decode the basic block starting at `start`, up to `max_insns`.
pub fn decode_block(mem: &Memory, start: u32, max_insns: usize) -> Result<Block, Fault> {
    let mut insns = Vec::new();
    let mut at = start;
    let mut ends_with_branch = false;
    while insns.len() < max_insns && !(GATE_BASE..GATE_END).contains(&at) {
        let i = decode(mem, at)?;
        insns.push((at, i));
        at = at.wrapping_add(i.len as u32);
        if i.ends_block() {
            ends_with_branch = true;
            break;
        }
    }
    Ok(Block {
        start,
        insns,
        end: at,
        ends_with_branch,
    })
}

/// Opaque handle to translated code owned by a translator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NativeHandle(pub u64);

pub trait Translator {
    fn name(&self) -> &'static str;
    /// Translate a block, or decline.
    fn translate(&mut self, block: &Block) -> Option<NativeHandle>;
    /// Code in `[page_start, page_start + len)` was written; drop any
    /// translation that read bytes from it.
    fn invalidate(&mut self, page_start: u32, len: u32);
}
