//! `interpret`'s `.afterok` and `.fail`: run or compile what a token was.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::{EDI, EDX};
use reva_core::{Dispatch, Gate, Machine, Next};
use reva_core_inspect::words::Dictionary;
use reva_words_code::emit::Emit;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    InterpIsWord => interp_is_word;
};

/// After `isword?`: its kind, or `.fail`: abandon a half-made
/// definition and leave `interpret`.
pub fn interp_is_word(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let kind = m.upop()?;
    if kind != 0 {
        return m.after_ok(kind);
    }
    m.rollback_definition()?;
    Ok(Next::Ret)
}

reva_macros::machine_trait! {
    /// What the outer interpreter does with a token it recognised.
    pub trait Outer: Emit {
        /// `interpret`'s `.afterok`: `ecx` is the token kind + 1 from the
        /// tokenizer (1 = dictionary entry, 2 = single, 3 = double).
        fn after_ok(&mut self, ecx: u32) -> R<Next> {
            let kind = ecx.wrapping_sub(1);
            if kind == 0 {
                return self.enter_word();
            }
            if self.compiling() {
                self.compile_number(kind)?;
            }
            Ok(Next::Jump(Gate::InterpWord.addr()))
        }

        /// Call the entry's class with EAX = xt, EDX = entry, EDI = class.
        fn enter_word(&mut self) -> R<Next> {
            let entry = self.eax();
            let class = self.mem.read_u32(entry.wrapping_sub(4))?;
            self.cpu.r[EDX as usize] = entry;
            self.cpu.r[EDI as usize] = class;
            self.set_eax(self.mem.read_u32(entry.wrapping_add(4))?);
            Ok(Next::Call(class, Gate::InterpWord))
        }

        /// A number while compiling: one literal, or two for a double.
        fn compile_number(&mut self, kind: u32) -> R {
            match kind {
                1 => {
                    let v = self.upop()?;
                    self.compile_literal(v)
                }
                2 => {
                    let hi = self.upop()?;
                    let lo = self.upop()?;
                    self.compile_literal(lo)?;
                    self.compile_literal(hi)
                }
                _ => Ok(()),
            }
        }
    }
}
