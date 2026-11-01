//! The data classes: constants, variables, values, `does>`, deferred.

use crate::classes::run_word;
use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::x86::EBX;
use reva_core::{Dispatch, Gate, Machine, Next};
use reva_words_code::emit::Emit;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    CClass => c_class;
    VClass => v_class;
    ValClass => val_class;
    DoesClass => does_class;
    DClass => d_class;
};

/// `'constant`: `cclass` has no `ret`: it falls through into `vclass`, so
/// while compiling the constant's value becomes a literal.
pub fn c_class(m: &mut Machine, g: Gate, h: &mut dyn Host) -> R<Next> {
    let v = m.mem.read_u32(m.eax())?;
    m.set_eax(v);
    v_class(m, g, h)
}

/// `'variable`: its address; a literal while compiling.
pub fn v_class(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    if m.compiling() {
        let v = m.upop()?;
        m.compile_literal(v)?;
    }
    Ok(Next::Ret)
}

/// `'value`: its contents; compiled as the address and a fetch.
pub fn val_class(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    if m.compiling() {
        let v = m.upop()?;
        m.compile_literal(v)?;
        m.compile_call(Gate::CClass.addr())?;
    } else {
        let v = m.mem.read_u32(m.eax())?;
        m.set_eax(v);
    }
    Ok(Next::Ret)
}

/// `does>`: xt-4 holds the address of the code after `does>`.
pub fn does_class(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.dup()?;
    let code = m.mem.read_u32(m.eax().wrapping_sub(4))?;
    m.set_eax(code);
    if !m.compiling() {
        return run_word(m);
    }
    let code = m.upop()?;
    let xt = m.upop()?;
    m.compile_literal(xt)?;
    m.compile_call(code)?;
    Ok(Next::Ret)
}

/// `'defer`: call through the cell (`call dword [xt]` while compiling).
pub fn d_class(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    if m.compiling() {
        let xt = m.upop()?;
        let mut b = [0xff, 0x15, 0, 0, 0, 0];
        b[2..].copy_from_slice(&xt.to_le_bytes());
        m.emit_code(&b)?;
        return Ok(Next::Ret);
    }
    let cell = m.upop()?;
    m.cpu.r[EBX as usize] = cell;
    Ok(Next::Jump(m.mem.read_u32(cell)?))
}
