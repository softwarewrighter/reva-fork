//! The dispatch table: every gate to its handler, checked when compiled.

use reva_core::fault::Fault;
use reva_core::host::Host;
use reva_core::{Dispatch, Gate, Machine, Next};
use reva_words_classes::{classes, data, defers, lookup};
use reva_words_compile::{compiling, numbers};
use reva_words_eval::{catch, define, eval, variables};
use reva_words_ffi::os;
use reva_words_interp::{interp, outer, startup};
use reva_words_memory::{lists, memory, strings};
use reva_words_system::{extra, system};

/// The number of gates.
const GATES: usize = Gate::ALL.len();

/// Every family's gates, as their modules list them.
const FAMILIES: &[&[(Gate, Dispatch)]] = &[
    classes::ENTRIES,
    data::ENTRIES,
    defers::ENTRIES,
    lookup::ENTRIES,
    startup::ENTRIES,
    interp::ENTRIES,
    outer::ENTRIES,
    eval::ENTRIES,
    catch::ENTRIES,
    define::ENTRIES,
    variables::ENTRIES,
    compiling::ENTRIES,
    numbers::ENTRIES,
    memory::ENTRIES,
    strings::ENTRIES,
    lists::ENTRIES,
    system::ENTRIES,
    extra::ENTRIES,
    os::ENTRIES,
];

/// The handler of each gate, by gate number.
pub const TABLE: [Dispatch; GATES] = build(FAMILIES);

/// The table from the families' lists.  Compilation fails if a gate is
/// listed twice or not at all.
const fn build(families: &[&[(Gate, Dispatch)]]) -> [Dispatch; GATES] {
    let mut table: [Dispatch; GATES] = [unlisted; GATES];
    let mut seen = [false; GATES];
    let (mut f, mut count) = (0, 0);
    while f < families.len() {
        let mut i = 0;
        while i < families[f].len() {
            let (g, handler) = families[f][i];
            assert!(!seen[g as usize], "a gate is listed twice");
            seen[g as usize] = true;
            table[g as usize] = handler;
            count += 1;
            i += 1;
        }
        f += 1;
    }
    assert!(count == GATES, "a gate has no handler");
    table
}

/// The placeholder `build` replaces for every gate.
fn unlisted(_: &mut Machine, g: Gate, _: &mut dyn Host) -> Result<Next, Fault> {
    Err(Fault::BadGate { addr: g.addr() })
}
