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

/// A check: its name, what it does, and its switch in the machine.
type Check = (&'static str, &'static str, fn(&mut Machine) -> &mut bool);

/// The checks.
const CHECKS: &[Check] = &[
    (
        "underflow-at-word",
        "stop at the word that leaves the stack below empty, with its callers",
        |m| &mut m.underflow_at_word,
    ),
    (
        "execute",
        "stop an execute (any indirect call or jump) of what is not code",
        |m| &mut m.check_execute,
    ),
    (
        "compile",
        "say what is compiled into a definition, and who compiled it",
        |m| &mut m.trace_compile,
    ),
];

/// `diagnose` ( "name" -- ).
pub fn diagnose(m: &mut Machine, _: Gate, host: &mut dyn Host) -> R<Next> {
    let (a, n) = m.parsews(host)?;
    let name = String::from_utf8_lossy(m.mem.slice(a, n)?).into_owned();
    match CHECKS.iter().find(|c| c.0 == name) {
        Some((_, _, flag)) => *flag(m) = true,
        None if name == "off" => CHECKS.iter().for_each(|(_, _, flag)| *flag(m) = false),
        None => host.write(usage(m).as_bytes()),
    }
    Ok(Next::Ret)
}

/// The checks, each marked if on.
fn usage(m: &mut Machine) -> String {
    let lines: Vec<String> = CHECKS
        .iter()
        .map(|(name, what, flag)| {
            let on = if *flag(m) { "*" } else { " " };
            format!("  {on} {name:<20} {what}")
        })
        .collect();
    format!("diagnose NAME (or off); * is on:\n{}\n", lines.join("\n"))
}
