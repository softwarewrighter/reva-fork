//! The compiling words.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_code::emit::Emit;
use reva_words_code::semi::Semi;
use reva_words_code::space::Space;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Literal => ret |m, _h| { let v = m.upop()?; m.compile_literal(v) };
    SSemi => ret |m, _h| m.ssemi();
    Semi => semi;
    InlineSemi => ret |m, _h| m.inline_semi();
    LBracket => ret |m, _h| m.lbracket();
    RBracket => rbracket;
    Compile => ret |m, _h| { let xt = m.upop()?; m.compile_call(xt) };
    Align => ret |m, _h| m.align_code();
    StoreHere => store_here;
    Comma | Comma3 | Comma2 | Comma1 => comma;
};

/// `;`: `;;` and `[`.
pub fn semi(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.ssemi()?;
    m.lbracket()?;
    Ok(Next::Ret)
}

/// `]`: one level deeper into compiling.
pub fn rbracket(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let s = m.var(st::IS_COMPILING);
    m.setv(st::IS_COMPILING, s.wrapping_add(1))?;
    Ok(Next::Ret)
}

/// `here,` ( a n -- ).
pub fn store_here(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    let n = m.upop()?;
    let a = m.upop()?;
    m.store_here(a, n)?;
    Ok(Next::Ret)
}

/// `,` `3,` `2,` `1,` ( x -- ).
pub fn comma(m: &mut Machine, g: Gate, _: &mut dyn Host) -> R<Next> {
    let n = match g {
        Gate::Comma => 4,
        Gate::Comma3 => 3,
        Gate::Comma2 => 2,
        _ => 1,
    };
    m.comma_n(n)?;
    Ok(Next::Ret)
}
