# local-quality

The local lane of reva-fork, worked on the user's Mac. The cloud agent
works the CoSy lane (reva-fork as CoSy's platform, plus
ba-cosy-zip-fork); this lane owns conformance, bugs and the features
below, and verifies everything on macOS, where the cloud cannot run.

Order, as decided with the user (2026-10-26): first sw-checklist
conformance, then bugs, then features.

Rules:
- src/reva.f stays verbatim (rust/reva-vm/src/embedded.rs inserts what
  the port needs); gate numbers are append-only (images store them).
- Refactors change no behaviour: every test, golden and the web smoke
  test pass unchanged, and the bench guard shows no slowdown beyond
  noise.
- The gate before every commit: `just check` (step 1 defines it).
  Until step 12, the known macOS failures (docs-samples order,
  parity-examples needs `timeout`) are listed in the gate and must not
  grow.
- One PR to dev per step, merged before the next structural step
  starts, so the cloud lane rebases onto small moves rather than one
  large one. Tell the user before moving files the cloud lane is
  editing.
- Commit .agentrail with the work; push; open the PR.

## Steps

### A. sw-checklist conformance (51 FAILs on 2026-10-26)

1. setup -- CLAUDE.md (+ AGENTS.md symlink): the agentrail briefing
   plus reva-fork's policy; `just check`, the one gate; the baseline
   (tests, goldens, sw-checklist counts) recorded.
2. edition-2024 -- both workspaces (rust/, desktop/) on edition 2024,
   rust-version 1.85; fix the fallout.
3. web-checklist -- reva-web: index.html and favicon.ico where
   sw-checklist looks, and a footer with copyright, license,
   repository, build host, commit and time (reva_vm::version) on the
   page and in the desktop app.
4. bench-guard -- a quick, repeatable timing guard from bench/ (vx86
   inner loop, boot, a fractal), `just bench-guard`, run before and
   after every split step.
5. crate-layout -- decide with the user how reva-vm (23 modules, max
   7) splits into crates; record it in rust/DESIGN.md; do the moves
   only (no code changes).
6. split-gates -- gates.rs: the 700-line `gate` dispatch and 51
   functions into family modules (functions <= 50 lines, modules <= 7
   functions, files <= 500 lines).
7. split-x86 -- decode.rs, exec.rs, fpu.rs (decode 381 lines, fpu_exec
   248, fpu_op 206, exec 194, shift, muldiv).
8. split-machine-fs -- machine.rs (build_dictionary as a table), vfs.rs,
   fileio.rs, mem.rs, heap.rs, host.rs, ffi.rs.
9. split-gfx -- canvas.rs, turtle.rs (turtle_call), highlight.rs.
10. split-cli -- reva-cli: main, lineedit (key, 576 lines), term,
    history, fs; the binaries recognised so the help/version checks run.
11. split-front-ends -- reva-web lib.rs, reva-serve http.rs,
    reva-desktop main.rs; then sw-checklist has 0 FAILs and `just
    check` requires it; warnings reviewed and listed.

### B. Bugs

12. portable-gates -- parity-examples.sh and desktop/smoke.sh without
    GNU `timeout`; docs-samples in a fixed order; `just check` fully
    green on macOS with no known-failure list.
13. web-ms -- `ms` really waits in the browser and the desktop app,
    without blocking the page.
14. dock-icon -- plain reva-desktop shows Reva's icon in the Dock
    (verified on the Mac).
15. license-tools -- the license check is complete locally: `just
    setup` installs or checks the tools (reg-rs, sed-rs, cargo-deny).

### C. Features

16. ci -- GitHub Actions on macOS and Linux running `just check`.
17. repl-polish -- `pp` without `needs highlight/core`; `reva -V`
    marks a dirty tree.
18. reva-app -- `just reva-app` builds ~/Applications/Reva.app.
19. examples-triage -- the 16 examples that fault or miss a word.
20. image-compat -- a policy for saved images across releases, and a
    fixture test.
21. perf-baseline -- benchmark numbers recorded, `just bench`, the
    reference for the translators.
22. live-demo -- GitHub Pages serving the web build, linked from the
    README.
23. release -- version bump, tags, a GitHub release (CLI binaries, web
    zip, macOS dmg; signing and notarization decided with the user).
