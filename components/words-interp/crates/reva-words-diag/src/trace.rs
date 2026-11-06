//! `diagnose compile`: say what is compiled into the definition being
//! built, and which words compiled it when it was not the interpreter
//! (CoSy's ask 22: a word that compiles where it should run).

use reva_core::host::Host;
use reva_core::{Machine, st};
use reva_core_inspect::source::Source;
use reva_core_inspect::words::Dictionary;

/// With the trace on, report that `xt` is compiled (`how`: "compile" for a
/// call, "inline" for copied bytes) into the newest definition.
pub fn trace_compile(m: &Machine, host: &mut dyn Host, xt: u32, how: &str) {
    if !m.trace_compile {
        return;
    }
    let word = m.describe_addr(xt);
    let word = word.strip_suffix("+0x0").unwrap_or(&word);
    let into = newest(m).unwrap_or_else(|| "?".into());
    let by = m.callers();
    let by = match by.is_empty() {
        true => String::new(),
        false => format!(" by {}", by.join(" <- ")),
    };
    host.diag(&format!("{how}: {word} into {into}{by}\n"));
}

/// The name of the newest dictionary entry: the definition being built.
fn newest(m: &Machine) -> Option<String> {
    let e = m.mem.read_u32(m.var(st::LAST)).ok()?;
    let n = m.mem.read_u8(e + 8).ok()? as u32;
    Some(String::from_utf8_lossy(m.mem.slice(e + 9, n).ok()?).into_owned())
}
