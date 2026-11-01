# Code metrics and structure

How reva-fork's Rust code is shaped: the limits, the rules behind them,
and what to do when a limit is reached. The rules are the user's
(2026-10-28), taken from how
[sw-mlpl](https://github.com/sw-ml-study/sw-mlpl) restructured into
components (its `docs/code_metrics.md` and `docs/loose-coupling.md`);
this is our version of them, with reva-fork's examples.
`sw-checklist` enforces its own, looser FAIL lines; ours are the target.

## Limits

| Part | Target (ours) | sw-checklist FAIL |
|---|---|---|
| Lines per function | 25 | over 50 (warning over 25) |
| Functions per module | 5 | over 7 |
| Lines per file | (follows from the above) | over 500 |
| Modules per crate | 4 | over 7 |
| Crates per component (Cargo workspace) | 4 | — |
| Components | as many as needed | — |

Each part should fit in one's head: about five things at a level, with
room to grow before the next split.

## Layout

```
components/<name>/          one Cargo workspace per component
  Cargo.toml                [workspace], [workspace.package], profiles
  Cargo.lock
  crates/<crate>/           at most 4 crates
    src/lib.rs              mod and use only
    src/<module>.rs         at most 4 modules
    tests/<topic>_tests.rs  the tests
```

Components depend on each other through path dependencies
(`path = "../../../core/crates/reva-core"`). They all build into the
repository's `target/` (`.cargo/config.toml`), share one version
(`scripts/check.sh` checks it) and one license policy (`deny.toml`).
The dependency graph has no cycles: x86 and host at the bottom, then
core, vlib, words, vm, and the front ends (cli, web, serve, desktop)
on top; sqlite stands beside them for the native hosts.

## Rules

1. **Move out, up and out; never combine.** When a part is full, split
   it: a new module, then a new crate in the component, then a new
   component. Never merge parts, shorten names or squeeze formatting
   to get under a limit.
2. **Facades.** `lib.rs` and `mod.rs` hold only `mod`, `use`/`pub use`
   and docs: no functions, no macros, no dispatch. Behaviour lives in
   named files.
3. **Pure functions.** Decide in pure functions (arguments in, value
   out); keep effects (memory writes, host calls, I/O) at the edges, in
   small functions that call them.
4. **Impls apart from structs.** A struct's definition stays in one
   place; its methods sit in `impl` blocks in sibling files, grouped by
   responsibility (Rust allows `impl` blocks anywhere in the crate; the
   `machine_trait!` macro in reva-macros lets other crates add methods
   to `reva_core::Machine`).
5. **Tests apart from code.** Tests go in `tests/*_tests.rs`, not in
   `#[cfg(test)]` modules inside `src/`, so test functions do not count
   against a module's budget and the public surface is what is tested.
6. **Patterns and delegation.** Facade (reva-vm over reva-core),
   strategy through function pointers or traits (the `Dispatch` fn
   pointer the run loop calls), chain of responsibility, builder;
   composition over one large type that does everything.
7. **Loose coupling.** A crate knows its neighbours by small interfaces
   (traits such as `Host`, plain data such as `HostCall`), not by their
   internals. Dependencies point one way.
8. **Boilerplate in macros, tables or build.rs.** Repetition is written
   once: a `macro_rules!` (`gates!`, `machine_trait!`), a table read by
   a generic function (the dictionary, key bindings), or code generated
   at build time (the embedded `lib/` tree).
9. **Measure before the change.** Count the functions and lines a step
   will add and design the split first, with headroom: a module at 5 of
   7 functions is full for planning purposes.

## Symptom → technique

### A function over 25 lines

| Symptom | First thing to try |
|---|---|
| Steps A, B, C done inline | One named helper per step; the function becomes the list of calls. |
| A `match` whose arms are long | Each arm a named function; the `match` only dispatches (a table if the arms are uniform). |
| Nested loops with an accumulator | An iterator chain (`map`, `filter`, `fold`, `collect`), each stage a small function. |
| The same checks at the top of many functions (stack depth, arguments) | A macro or a helper that does the checks once. |
| Parse, check and emit mixed | `parse_*`, `check_*`, `emit_*`, chained by a short function. |
| Nested `match` on `Result`/`Option` | `?`, `and_then`, `map_err`. |

### A module over 5 (FAIL: 7) functions

| Symptom | First thing to try |
|---|---|
| Methods that cluster (memory, stack, I/O) | One sibling file per cluster, each its own `impl` block. |
| Many functions of the same shape | A macro or a table; the functions disappear into it. |
| Tests in the module | Move them to `tests/*_tests.rs`. |
| A dispatcher with its arm handlers | Dispatcher in one file, handlers in another (reva-words: `dispatch.rs`, the gate families). |

### A file over 500 lines

| Symptom | First thing to try |
|---|---|
| Data (tables, word lists) | Split by topic into sibling files, or move the data out of Rust (a file read by build.rs). |
| One type with many methods | `impl` blocks in sibling files, by responsibility. |
| Dispatch, handlers and tests together | One file per role. |

### A crate over 4 (FAIL: 7) modules

| Symptom | First thing to try |
|---|---|
| Modules on different topics | Move a topical group into a new crate in the same component; the old crate re-exports it if callers need the old path. |
| The component already has 4 crates | A new component, with its own workspace. |

### A clippy `allow`

Fix the code the lint points at; do not silence it.

## When stuck

Most oversized code mixes phases: work that could happen at compile
time (a macro, a const table), at start-up (building the dictionary),
or once per input (decoding) done again inside a hot loop or written
out by hand. Find which phase each piece belongs to and move it there;
the function usually shrinks on its own.
