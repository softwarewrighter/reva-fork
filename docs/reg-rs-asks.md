# What reva-fork needs from reg-rs

Requests from running this repository's goldens (`work/reg-rs/`, through
`scripts/regress.sh`) with [reg-rs](https://github.com/sw-cli-tools/reg-rs),
most valuable first. Each has the case that hit it and what would fix it.

## 1. Build and run on Windows (filed 2026-11-22, step local-ci; done)

**Answered** by [reg-rs #1](https://github.com/sw-cli-tools/reg-rs/pull/1)
(merged 2026-11-22, 1a18986): it builds and runs on Windows, `file-lock`
replaced, and reg-rs has CI on Linux, macOS and Windows. reva-fork's CI
installs it from main (`--locked`) on all three and runs the goldens
there. Also fixed on the way: a test name with a folder (`subdir/foo`)
kept as typed on Windows, not turned into `subdir\foo.rgt`.

The ask as filed:

reva-fork's CI runs the gate on `windows-latest` (CoSy's ask 28: Bob's
users are Windows users), and the gate runs the 70 goldens. reg-rs does
not build there, and with `--locked` it does not build on macOS either:

```text
cargo install --locked --git https://github.com/sw-cli-tools/reg-rs
  windows-latest: error: failed to run custom build command for `file-lock v1.1.20`
                  (it compiles C that uses fcntl, which Windows lacks)
  macos-latest:   error[E0425]: cannot find value `NOTE_EXIT_REPARENTED` in crate `libc`
                  (nix 0.11.1, from file-lock, against the locked libc 0.2.191)
```

Both come from one dependency, `file-lock` (reg-rs-store's `db.rs`), and
through it `nix` 0.11 (2018).

- **Replace `file-lock` with std's `File::lock` / `File::lock_shared`**
  (stable since Rust 1.89): the same advisory lock, on Unix and Windows,
  with no C and no `nix`.
- **Kill a timed-out test with `Child::kill`**, not `kill -9 PID`
  (`process_io.rs`, `kill_child`): Windows has no `kill`, and Git Bash's
  `kill` does not take a Windows process id.
- **Keep running commands through `sh -c`.** On Windows that is Git
  Bash's `sh`, which is how reva-fork runs everything there, so the
  `.rgt` commands stay the same on every OS. Say so in the README.
- **CI for reg-rs on `windows-latest`** (and macOS, Linux) building with
  `--locked`, so a break like this shows up in reg-rs and not in its users.

SQLite: reg-rs-store's `rusqlite` 0.30 links the system's SQLite (no
`bundled` feature), like reva-fork's: on Windows the official
`sqlite3.dll` and an import library made from its `.def`, which
reva-fork's `scripts/sqlite-dll.sh` provides; `SQLITE3_LIB_DIR` points at
them and the DLL's folder is on `PATH`.

**Why:** without it the Windows job cannot run the goldens, the tests
that pin down what Reva prints; the cargo tests alone do not cover them.
