//! List words: `link`, `-link`, `onexit`, `onstartup`, `iterate`.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next, st};
use reva_words_calls::lists::Lists;
use reva_words_calls::vectors::Vectors;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Link => ret |m, _h| m.link();
    Link2 => ret |m, _h| m.link_end();
    OnExit => on_exit;
    OnStartup => on_startup;
    Iterate => iterate;
};

/// `onexit` ( xt -- ): run at `bye`.
pub fn on_exit(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.upsh(st::EXIT_LIST)?;
    m.link()?;
    Ok(Next::Ret)
}

/// `onstartup` ( xt -- ): run at `cold`, after those before.
pub fn on_startup(m: &mut Machine, _: Gate, _: &mut dyn Host) -> R<Next> {
    m.upsh(st::START_LIST)?;
    m.link_end()?;
    Ok(Next::Ret)
}

/// `iterate` ( xt list -- ).
pub fn iterate(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let list = m.upop()?;
    let xt = m.upop()?;
    m.iterate_list(host, xt, list)?;
    Ok(Next::Ret)
}
