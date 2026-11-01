//! The code classes: `'forth`, `'macro` (and their `notail` forms),
//! `'inline`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::{EBX, EDX};
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_code::emit::Emit;
use reva_words_code::space::Space;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    FClass | FClassNotail => f_class;
    MClass => m_class;
    MClassNotail => m_class_notail;
    IClass => i_class;
};

/// `'forth`: compile a call while compiling, else run the word.
pub fn f_class(m: &mut Machine, g: Gate, _: &mut dyn Host) -> R<Next> {
    m.setv(
        st::SSEMI_DO_TAIL,
        if g == Gate::FClass { 0x90 } else { 0xc3 },
    )?;
    if !m.compiling() {
        return run_word(m);
    }
    let xt = m.upop()?;
    m.compile_call(xt)?;
    Ok(Next::Ret)
}

/// `'macro`: run the word, compiling or not.
pub fn m_class(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    run_word(m)
}

/// `'macro` that must not end in a tail call.
pub fn m_class_notail(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.setv(st::SSEMI_DO_TAIL, 0xc3)?;
    run_word(m)
}

/// `'inline`: copy the word's bytes into the definition.
pub fn i_class(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.setv(st::SSEMI_DO_TAIL, 0xc3)?;
    if !m.compiling() {
        return run_word(m);
    }
    let xt = m.upop()?;
    let n = m.mem.read_u8(xt.wrapping_sub(1))? as u32;
    m.cpu.r[EDX as usize] = xt;
    m.store_here(xt, n)?;
    Ok(Next::Ret)
}

/// `mclass`: jump to the xt on the stack, with EBX = xt.
pub fn run_word(m: &mut Machine) -> R<Next> {
    let xt = m.upop()?;
    m.cpu.r[EBX as usize] = xt;
    Ok(Next::Jump(xt))
}
