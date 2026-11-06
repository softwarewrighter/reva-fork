//! `diagnose NAME`: turn on a check (`off` turns them all off; no name, or
//! one it does not know, lists them and what is on).

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next};
use reva_words_scan::parse::Parse;

type R<T = ()> = Result<T, Fault>;

/// The gates this module serves.
pub const ENTRIES: &[(Gate, Dispatch)] = reva_macros::words! {
    Diagnose => diagnose;
};

/// The checks: name, what it does.
const CHECKS: &[(&str, &str)] = &[(
    "underflow-at-word",
    "stop at the word that leaves the stack below empty, with its callers",
)];

/// `diagnose` ( "name" -- ).
pub fn diagnose(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let (a, n) = m.parsews(host)?;
    let name = String::from_utf8_lossy(m.mem.slice(a, n)?).into_owned();
    match name.as_str() {
        "underflow-at-word" => m.underflow_at_word = true,
        "off" => m.underflow_at_word = false,
        _ => host.write(usage(m).as_bytes()),
    }
    Ok(Next::Ret)
}

/// The checks, each marked if on.
fn usage(m: &Machine) -> String {
    let on = |name: &str| name == "underflow-at-word" && m.underflow_at_word;
    let lines: Vec<String> = CHECKS
        .iter()
        .map(|(name, what)| format!("  {} {name:<20} {what}", if on(name) { "*" } else { " " }))
        .collect();
    format!("diagnose NAME (or off); * is on:\n{}\n", lines.join("\n"))
}
