# Changes

Every commit of the fork, newest first, grouped by day. Times are
Pacific (UTC-07:00), as committed. The fork starts on 2026-10-24; the
original Reva history (imported from its fossil repository as edc8cbe,
2020) is not listed.

Categories: `feat` new capability, `fix` a bug or wrong behavior,
`refactor` structure without behavior change, `test` tests only,
`build` build, tooling and gates, `docs` documentation, `design` a
decision recorded, `plan` saga planning and reordering, `release` a
version, `chore` bookkeeping (saga records, branch syncs).

## 2026-10-27

- 22:23 `chore` Step local-crate-layout complete (saga record).
- 22:23 `refactor` The first crate split (step local-crate-layout; the user chose full sw-checklist conformance, rust/DESIGN.md 3.12): new crate reva-x86 (the CPU types, decode, exec, fpu, mem, fault, layout) and reva-host (host, hostcall, vfs, embedded, version, charset, and the build script that embeds lib/ and stamps the build), by pure moves (`git mv`); reva-vm re-exports both under the old paths, so no host or test changed. reva-vm: 27 modules to 14; reva-x86 and reva-host 7 each. Same 99 tests, every golden unchanged, bench guard within +3%. Doc links that now cross crates are code spans. DESIGN 3.12 records the plan (reva-core with a dispatch table, reva-words, reva-vlib); the saga's split steps rewritten for it. rust/README.md layout.
- 21:58 `chore` Step local-bench-guard complete (saga record).
- 21:58 `build` The timing guard for refactors of hot code (step local-bench-guard): `just bench-guard` (rust/scripts/bench-guard.sh) times four workloads on the release CLI, the median of five runs each (boot: ten launches of all of reva.f; fib 30: calls and returns; a fetch in a tight do-loop, bench/bench.f's `dl`; twelve dragon curves: turtle gateways), and compares them with this host's line in work/bench-guard.txt, failing over 15% slower (the run-to-run spread here is up to about 10%). `--record` records a host's baseline. Baseline on this Mac (max): boot 505 ms, fib 1066 ms, loop 1252 ms, dragon 405 ms; three later runs within +9%. Not in `just check` (about 17 s, noisy on a busy machine): the split steps run it before and after.
- 21:44 `feat` Merge #48 (local-iconv): a virtual libiconv inside the VM.
- 21:43 `chore` Step local-iconv complete (saga record).
- 21:43 `feat` iconv (step local-iconv, at the user's request): the virtual library `libiconv` for lib/string/iconv, unchanged: `libiconv_open`, `libiconv`, `libiconv_close` and `_libiconv_version`, done inside the VM on every host with the standard library only (reva-vm charset.rs: UTF-8, UTF-16, UCS-2, UTF-32 in both byte orders, Latin-1, ASCII; iconv.rs: descriptors, the in/out pointer cells advanced in VM memory as C's iconv does, -1 with errno E2BIG, EILSEQ, EINVAL, EBADF). Gates IcOpen, Iconv, IcClose appended; static cell ICONV_VERSION. examples/utf8.f and notready/normalize.f now run (73 of 132); the 21 GUI programs that stopped at iconv now stop at IUP (`IupOpen`, and one at its CD library). Tests: charset tests (tests/charset.rs), golden iconv; goldens parity-examples rebased. Docs: portability, parity, x86-plan.
- 21:03 `feat` Merge #47 (local-sqlite-browser): SQLite in the browser.
- 21:03 `feat` Merge #46 (local-see): `see` shows a word's x86 again; docs/x86-plan.md.
- 20:58 `chore` Step local-sqlite-browser complete (saga record).
- 20:58 `feat` SQLite in the browser (step local-sqlite-browser): the VM's worker answers the `sqlite` host calls itself (rust/reva-web/www/sqlite.js), with SQLite's official WebAssembly build loaded at run time from jsDelivr, pinned to @sqlite.org/sqlite-wasm 3.54.1-build1, the same SQLite as the CLI's; nothing of it is redistributed (the user's choice over vendoring). Each database runs in memory, is loaded from the browser's private file system (OPFS) on open and written back after every committed change, so it survives reloads; SQLite's own OPFS VFSes were rejected (`opfs` needs headers GitHub Pages cannot send; `opfs-sahpool`'s exclusive handles let a page in the back/forward cache block the next one, seen in Chrome). Offline: "SQLite could not be loaded". Pages pick worker-local SQLite with `localServices` (the desktop keeps its native answers). Attribution: About dialog, README "Third-party software", docs/licenses.md "Loaded at run time" (and what vendoring would require). Checked in Chrome: a table survives back-to-back reloads. Tests: web smoke (the worker answers sqlite itself; golden web-smoke). Docs: sqlite.md, web.md, DESIGN 3.11. Saga: step local-iconv added after this one at the user's request.
- 20:29 `chore` Step local-see complete (saga record).
- 20:29 `feat` `see` shows a word's x86 again (step local-see; CoSy's ask 3a): vx86 decodes `cbw` and `cwd` (`66 98`, `66 99`), which Reva's disassembler (lib/util/disasm, after `needs debugger`) uses, so it no longer faults on its first instruction and no longer leaves `base` in hex. lib/debugger: a core routine written in Rust (its code is a gateway, all `hlt` bytes) is reported as one instead of being disassembled forever. `see` over every word of the booted dictionary (356): no fault, no runaway, base decimal after. Tests: an executor test for cbw/cwd, golden see (a colon word, a call and a tail call, a constant, a variable, a core routine; addresses masked). docs/x86-plan.md (the user's questions): how vx86 runs Reva's x86 and the x87, what works, `see`, what breaks in legacy programs by cause with counts from the parity report, how each gets ported, and the plan.
- 10:25 `feat` Merge #45 (local-sqlite-desktop): SQLite in the desktop app.
- 10:24 `chore` Step local-sqlite-desktop complete (saga record).
- 08:35 `feat` SQLite in the desktop app (step local-sqlite-desktop): the wasm module hands every host call to the page (`reva_run` returns HOST_CALL; `reva_host_call_ptr`/`_len` give it as JSON; `reva_resume_int`, `reva_resume_bytes`, `reva_refuse` answer; reva-vm `Machine::refuse` and `BufferHost::host_calls`). reva.js: `Reva.hostCall`/`resume`, sessions take `onHostCall`; the worker posts `hostcall` and resumes on `hostreply`. The desktop's host.js answers through the new Tauri command `host_call` (reva-desktop/src/sql.rs) from reva-sqlite, databases confined to the app's folder like the file words; the plain browser refuses with "SQLite is not available in the browser yet". Checked in the desktop app: a table written in one run is read in the next, and macOS's sqlite3 reads the file. Tests: a C-ABI round trip in reva-web, seven web smoke checks (golden web-smoke), three desktop smoke checks (persisting, a real SQLite file, a name outside the folder refused). Docs: sqlite.md, desktop.md, portability.
- 00:08 `docs` Merge #44 (local-readme-demo): the live demo and its screenshot at the top of the README.
- 00:08 `chore` Merge #43 (step local-live-demo complete; the live demo published).
- 00:07 `chore` Step local-readme-demo complete (saga record).
- 00:07 `docs` README: the live demo at the top, under the title (a prominent link, the dragon and tree examples) with an inline screenshot, images/live-demo.png, of the live demo after `needs turtle/fractals 12 tree-demo` (the tree at its maximum depth, split view; it links to that example). Step inserted at the user's request.
- 00:01 `chore` Step local-live-demo complete (saga record): the live demo checked in Chrome (boots, dragon curve in the split view, footer names main 7030283, no console errors).

## 2026-10-26

- 23:58 `release` The live demo published: `just publish` from main pushed gh-pages (4aa1b13, the plain reva-fork site as one orphan commit); GitHub turned Pages on from that branch by itself, at https://softwarewrighter.github.io/reva-fork/.
- 23:57 `release` Merge dev into main (7030283, pushed directly at the user's request): everything since #17, gated with `just check` first.
- 23:52 `build` Merge #42 (local-live-demo): the live demo's publisher.
- 23:52 `build` The live demo's publisher (step local-live-demo): `just publish` (rust/scripts/publish-pages.sh) builds the plain reva-fork site with make-web-zip.sh (no package, split view), adds .nojekyll and force-pushes it as one orphan commit to gh-pages for GitHub Pages ("Deploy from a branch", no Actions); only from a clean HEAD on origin/main, or `--dry-run` from anywhere. README links https://softwarewrighter.github.io/reva-fork/ with fractal examples; docs/web.md says how it works and that all softwarewrighter.github.io pages share one localStorage origin.
- 23:44 `feat` Merge #41 (local-sqlite): SQLite on the CLI through host calls.
- 23:42 `chore` Step local-sqlite complete (saga record).
- 23:42 `feat` SQLite on the CLI, through host calls (step local-sqlite; rust/DESIGN.md 3.11 rewritten as built). New crate reva-sqlite drives SQLite with rusqlite's safe API (no C calls or `unsafe` of ours; libsqlite3-sys compiles SQLite into the binary): handles are small integers, a statement buffers its rows on its first step. reva-vm gains host calls (`reva_vm::hostcall`: `Host::host_call` answers now, later or not at all; `Status::HostCall`, `Machine::pending_host_call`, `Machine::resume`; nothing is popped until the answer is in) and the virtual library `reva-sqlite` (appended gates SqRc..SqPtr), with no dependency added. lib/db/sqlite rewritten on its pointer-free functions (`rsql_open` ... `rsql_ptr`; `sql_fetch` walks rows with `rsql_next`, running the callback on a stack of its own as SQLite's C callback did); every public word keeps its stack effect. reva-cli answers at once, mapping database paths as the file words do. sqlite2.f and sqlite3.f run (71 of 132 examples); sqlite.f and zipdb.f still stop on `value`, db-report.f on `0L println`. Tests: reva-sqlite integration tests, host-call pause/resume and "not available" machine tests, golden sqlite; goldens parity-examples and docs-samples updated. parity-examples.sh strips BSD wc's padding, so with GNU timeout installed it passes on macOS (removed from the gate's known failures). Workspace rust-version 1.89 (rusqlite), clippy's let-chain suggestions applied. Docs: sqlite.md, portability, parity, licenses (rusqlite and its MIT/Apache/Zlib dependencies). Saga: the key-value step becomes local-sqlite-browser (SQLite's wasm build, the user's choice), local-live-demo moved up to follow this step.
- 21:25 `docs` Merge #40 (local-changes): CHANGES.md.
- 21:23 `chore` Step local-changes complete (saga record).
- 17:18 `docs` CHANGES.md: every commit of the fork from 2026-10-24 on, newest first, grouped by day, in X_eTaL's format (the 2020 import of the original Reva history left out); CLAUDE.md: every commit adds its line here. Step inserted into the saga at the user's request.
- 15:55 `design` Merge #39 (local-sqlite-design): host calls; SQLite on the CLI and in the desktop app, a key-value store in the browser.
- 15:51 `chore` Step local-sqlite-design complete (saga record).
- 15:51 `design` SQLite, decided with the user (rust/DESIGN.md 3.11, docs/portability.md): SQLite on the CLI and in the desktop app through a new crate, reva-sqlite, over libsqlite3-sys with its bundled amalgamation (reva-vm stays dependency-free; rust-version to 1.88); a key-value store on every host, over localStorage in web pages; one mechanism, the host call, which pauses the VM with a typed request (Status::HostCall) and resumes it with the answer (the CLI at once, the desktop's wasm VM through the worker, the page and a Tauri command); `sqlite3_exec`'s Forth callback as a prepare/step loop. Saga: steps local-sqlite (CLI and the mechanism), local-sqlite-desktop and local-kv-store.
- 14:57 `chore` Step local-sqlite-design begun (saga record).
- 14:56 `plan` SQLite moved ahead of the checklist splits at the user's request: steps local-sqlite-design and local-sqlite inserted.
- 14:56 `feat` Merge #38 (local-web-checklist): the footer and reva-web's sw-checklist checks.
- 14:55 `chore` Step local-web-checklist complete (saga record).
- 14:55 `feat` A footer on the REPL page and in the desktop app: copyright, MIT License, a source link (the desktop opens it in the system browser), version, commit, build host and build time, with the full license and exact time on hover. The facts come from the wasm module (`reva_build_info_ptr`/`_len`, `reva_vm::version::fields()`), so they match `reva -V`; the worker passes them with its boot status and sessions expose `build`. rust/reva-web/index.html and favicon.ico are symlinks into www/, where sw-checklist looks (the user's choice). Web smoke test checks the facts; sw-checklist 49 -> 47 FAILs, every reva-web check passes.
- 14:31 `release` Merge #37: version 2026.1026.0.
- 14:30 `chore` Step local-version-2026-1026 complete (saga record).
- 14:29 `release` Version 2026.1026.0 (rust/Cargo.toml); the banner where it is spelled out (web smoke test, getting-started, demo.md) and the goldens boot-progress and web-smoke rebased. CLAUDE.md: the day's version stays `.0` all day, `.1` and up only for another release to main the same day, bumped only when the user asks. Step inserted into the saga at the user's request.
- 14:26 `build` Merge #36: Rust edition 2024.
- 14:18 `chore` Step local-edition-2024 complete (saga record).
- 14:18 `build` Both workspaces (rust/, desktop/) on edition 2024, rust-version 1.85: `cargo fix --edition` (`#[unsafe(no_mangle)]` wasm exports, `unsafe extern "C"` for termios, `unsafe` `env::set_var` with a SAFETY note), rustfmt's 2024 style. No behavior change; timing unchanged within noise. sw-checklist 51 -> 49 FAILs.
- 14:01 `build` Merge #35: the local-quality saga, CLAUDE.md/AGENTS.md and `just check`.
- 13:04 `chore` Step local-setup complete (saga record).
- 13:04 `build` CLAUDE.md (the agentrail briefing plus reva-fork's policy: the local and cloud lanes, the invariants, the gate, the workflow) with AGENTS.md as a symlink; `just check` (rust/scripts/check.sh), the one gate: fmt, clippy `-D warnings` and tests in both workspaces, the wasm build and node tests, the goldens, the license check and sw-checklist, with goldens known to fail listed per OS and a ceiling on sw-checklist FAILs, both only shrinking.
- 12:49 `plan` Saga local-quality, the local lane (worked and verified on the Mac while the cloud lane works CoSy): 23 steps, sw-checklist conformance first, then bugs, then features.

## 2026-10-25

- 18:57 `feat` Merge #34: the reva-fork logo.
- 18:56 `feat` The reva-fork "fork" logo (Reva's gear with a fork) for the desktop app, the favicon and the README: desktop/make-icons.sh makes icon.png, icon.ico and a new icon.icns from a transparent master cut out of the source picture.
- 16:20 `feat` Merge #33: package icon as the macOS Dock icon; a favicon for the web page.
- 16:19 `feat` The blue Reva gear as the web page's favicon (favicon.png and .ico from the desktop icon by rust/scripts/make-favicon.sh), shipped by the web zip, the desktop app and reva-serve.
- 16:19 `feat` A package's icon is also the macOS Dock icon while the app runs (NSApplication's applicationIconImage, set on RunEvent::Ready).
- 16:09 `fix` Merge #32: the REPL keeps the view at the bottom.
- 16:09 `fix` The REPL page scrolled before coloured output was added, so a `pp` listing ended below the visible area; it now scrolls once after each write, only if the view was at the bottom, and always on a submitted line.
- 16:02 `fix` Merge #31: forth-highlight.py's default tables.
- 16:02 `fix` forth-highlight.py finds lib/highlight/forth.tbl from a package checkout too (beside ../reva-fork or via `$REVA_FORK`), then every `*.tbl` beside the script.
- 16:01 `feat` Merge #30: `pp` and `hl-type`.
- 16:01 `feat` `needs highlight/core`: `pp FILE` (a file, else a library such as `pp turtle/fractals`) and `hl-type` ( a n -- ) list Forth source in colour; `slurp type` stays plain. The tokenizer is the virtual library `highlight` (gates HlType, HlColor appended), with one token table (lib/highlight/forth.tbl) shared with forth-highlight.py. The CLI colours at a terminal (`--color auto|always|never`, NO_COLOR, TERM=dumb); the web page renders the escapes as themed spans (www/ansi.js). Goldens highlight, highlight-plain; docs/highlight.md.
- 15:48 `fix` Merge #29: license-check under bash 3.2.
- 15:48 `fix` license-check.sh failed to parse under macOS's bash 3.2 (a here-document with an apostrophe inside `$( )`); it reads the reviewed list with `read -r -d ''` instead.
- 15:45 `feat` Merge #28: forth-highlight.py.
- 15:40 `feat` rust/scripts/forth-highlight.py: colour-highlighted Reva and CoSy source as a self-contained HTML page, or PNG through headless Chrome (Python standard library only); used for the blog's figures of lib/turtle/fractals.
- 14:09 `feat` Merge #27: line editing in the CLI REPL.
- 14:08 `feat` The CLI edits each line at a terminal: Up/Down history (~/.reva_history or `$REVA_HISTORY`), Ctrl-R/S search, the Emacs keys of the web page; raw mode only while a line is edited, piped input unchanged, `--no-edit` or TERM=dumb for plain lines. No crates (termios via an opaque buffer on Unix, SetConsoleMode on Windows). docs/repl.md.
- 14:00 `feat` Merge #26: history and line editing in the web REPL.
- 13:59 `feat` The web REPL (browser, desktop app, web zips, reva-serve): Up/Down and Ctrl-P/N history kept per package in localStorage, Ctrl-R/S search, Emacs editing keys; test/lineedit.mjs.
- 13:42 `feat` Merge #25: About wording, package build line, header hint.
- 13:42 `feat` About describes reva-fork as "Reva Forth on a portable 32-bit VM (vx86), for 64-bit hosts and WebAssembly"; a package's about may carry its own build; a manifest's "hint" replaces Reva's header hint.
- 13:19 `feat` Merge #24: the About dialog.
- 13:18 `feat` An About dialog (www/about.js): the package's entry from reva-package.json, then reva-fork's credits, links, licenses and build stamp (the `reva-build` meta tag from rust/scripts/build-stamp.sh, stamped by make-web-zip.sh and desktop/build.sh); the desktop opens links in the system browser.
- 13:10 `feat` Merge #23: reva-serve.
- 13:10 `feat` reva-serve: the browser REPL (and optionally a package) as one local binary with the site embedded, serving 127.0.0.1 on a fixed port so files persist, opening the browser, recognising a running copy; rust/scripts/build-serve.sh builds mac (universal), linux (static musl) and windows binaries. docs/serve.md.
- 13:04 `feat` Merge #22: split view, packages in the browser, the web zip.
- 13:04 `feat` The browser's split view (`?canvas=split`: REPL beside the graphics, a resizable, collapsible divider), `?package=DIR/` (reva-package.json's files and start files, title, icon, canvas), canvas.html redrawn sharp on resize, and `just web-zip` (rust/scripts/make-web-zip.sh), a reproducible static-site zip, optionally with a package. docs/web.md.
- 12:31 `feat` Merge #21: the x87 floating-point unit.
- 11:59 `test` docs-samples golden lists docs/turtle.md (added on dev by #20).
- 11:56 `chore` Merge origin/dev into the x87 branch.
- 11:56 `feat` vx86 runs x87 code (D8-DF): an FPU in the CPU with eight f64 registers as a stack with tags, control and status words, every memory format (80-bit exact on store), arithmetic, compares, constants and the transcendentals; fldenv/fsave and BCD are not provided. lib/math/floats passes its 35 tests on the CLI and in WebAssembly (still no imports); 69 of 132 examples run. DESIGN.md 3.10.
- 11:40 `feat` Merge #20: turtle graphics.
- 11:36 `feat` Turtle graphics: the turtle and its display list in the VM (canvas.rs), served as the virtual library `turtle`; lib/turtle/core (forward, right, penup, color, step/scale, tpush/tpop, svg-save ...) and lib/turtle/fractals (Koch, Sierpinski, dragon, Hilbert, tree, an L-system plant). Drawn in a popup or pane in the browser, a second window in the desktop app, SVG on the CLI. Golden turtle; docs/turtle.md.
- 11:36 `feat` Merge #19: packages brand the desktop window.
- 11:31 `docs` Merge #18: demo script, packaging and Windows guide; Windows file paths.
- 11:29 `feat` reva-package.json's `title`, `icon` and `id` (Wayland app_id, X11 WM_CLASS) brand the desktop window at run time; the main window is created in setup() so it opens with the package's title and icon.
- 10:47 `docs` docs/demo.md (the CLI, the browser, the desktop app and a package, with executed samples) and docs/packaging.md (redistributable builds, the web zip, installers and signing, license notices, Windows builds and a test plan).
- 10:47 `build` Desktop: icon.ico for Windows builds, NSIS and MSI installers, license files in every installer; smoke.sh's restart check expected the old banner.
- 10:47 `fix` reva-cli on Windows: host paths (C:\dir) are shown to the VM as /C:/dir and mapped back, so include, slurp and relative file words work; checked under Wine.
- 10:22 `build` Merge #13: scripts use sed-rs.
- 10:21 `build` doc-check.sh and parity-examples.sh filter text with sed-rs instead of the system sed (they depended on GNU sed); doc-check.sh strips NUL bytes from reva's output.
- 10:17 `feat` Merge #15: version 2026.1025.0, `reva -V`, the `version` word, the reva-fork banner.
- 10:17 `chore` Merge dev into the version branch: the Version gate goes after HostSystem (gates are appended); core-errors, native-traps and parity-examples rebased for the fixed +0x14 shift of fault addresses.
- 10:15 `fix` Merge #16: restart includes a page's start files again.
- 10:14 `fix` Forth's `restart` in the desktop app and the browser left plain Reva (CoSy's start files were not included again); `reva_feed_start` keeps start input and replays it after `restart`, also in a worker replaced after Stop.
- 10:04 `feat` Merge #14: the desktop app.
- 10:03 `feat` Version 2026.1025.0 (YYYY.MMDD.N): `reva -V` / `--version` and the core word `version` print the version, copyright, license, repository and build host, commit and time (reva-vm/build.rs); the banner is `reva-fork 2026.1025.0 portable` (src/reva.f stays verbatim: a new `.ver` is inserted at boot); `revaver` still answers 2011.2. `version` is a gate, so build details do not change the dictionary's size.
- 10:03 `feat` The desktop app (desktop/, its own workspace): the Reva REPL in a Tauri window running the same wasm VM, files in a folder (~/Reva, `--root`, `--package`), packages with start files, `-e` and `--print-output` batch runs, `bye` closes the window; desktop/build.sh and smoke.sh; cargo-deny in both workspaces. docs/desktop.md.
- 09:48 `feat` Merge #11: clear stops for native libraries, shell and unsupported x86 code.
- 09:48 `chore` Merge origin/dev into the native-stops branch (parity.md, main.rs, gates.rs, machine tests).
- 09:41 `chore` Merge origin/dev into the native-stops branch.
- 09:41 `feat` Merge #10: saved images; restart.
- 09:12 `chore` Merge origin/dev into the images branch.
- 09:11 `fix` Merge #12: vx86 mov AL/EAX <-> [moffs32].
- 09:05 `fix` vx86 decodes mov AL/EAX <-> [moffs32] (A0-A3), which lib/util/locals compiles for every `local[` word: words with locals faulted, and CoSy.f stopped loading in Furniture.f.
- 08:50 `feat` Clear stops: a native library the core does not serve gets a one-line note naming it; `(call)` of a non-gateway address is Fault::NativeCall; unsupported instructions name their family (x87, SSE, privileged ...); C `system()` runs only with the CLI's `--allow-shell`. docs/portability.md; golden native-traps.
- 08:49 `chore` Merge dev (license gate) into the images branch.
- 08:43 `feat` Saved images: `save NAME` writes a vx86 image through the host's files; the browser starts from an image (`?image=`, `reva_boot_image`); `restart` is a core word, and hosts make a new machine the same way and hand over unread input. 64 of 132 examples run. DESIGN.md 3.9.
- 08:31 `build` Merge #9: copyleft files removed; the license gate.
- 08:24 `build` GPL and LGPL files removed from the tree (PolarSSL SHA-1 sources, the MySQL client, gpgme, gpg-error, libiconv, pthreads-win32); docs/licenses.md, rust/deny.toml and rust/scripts/license-check.sh, which fails on GPL-family text, unreviewed binaries and cargo-deny findings. The GUI and database decisions recorded in docs/parity.md.
- 08:00 `docs` Merge #7: the dev/main branch policy.
- 07:59 `feat` Merge #8: the browser VM in a Web Worker.

## 2026-10-24

- 19:57 `feat` The browser runs the VM in a Web Worker (www/reva-worker.js) in time slices of about 8 ms, with a Stop button and Ctrl-C (Machine::interrupt); a worker that does not answer in 2 s is replaced; output drawn once per animation frame. DESIGN.md 3.8.
- 19:26 `docs` The branch policy: dev is the default branch, main the tested release line; the original import is named by its commit (edc8cbe), not by master.
- 19:25 `feat` Merge #6: file words and file system calls; browser files in localStorage.
- 19:25 `chore` Merge dev (README rewrite) into the file-words branch.
- 19:23 `feat` File words and file system calls through Host::fs: open/r, open/rw, creat, close, read, write, fsize and the Linux file syscalls, `ioerr` as newcore had it; MemFs in the browser with written files kept in localStorage; lib/ embedded and read-only at /reva/lib/ on every host. DESIGN.md 3.7.
- 19:23 `feat` vx86 decodes and runs bswap r32 (lib/crypt/sha1 computes correct digests).
- 19:05 `docs` Merge #4: README rewritten for the portable core.
- 18:57 `docs` The original README kept as orig-README; README.md rewritten for the portable core (the vx86 approach, hosts, the roadmap and the CoSy-on-WASM goal), with the copyright line "Copyright (c) 2026 Michael A. Wright" across the license files.
- 18:54 `feat` Merge #3: all of reva.f boots.
- 18:53 `docs` Merge #2: the old wiki deprecated; docs/ rebuilt with tested samples.
- 18:53 `refactor` Merge #1: the deprecated x86 implementation under legacy/.
- 17:54 `feat` All of reva.f boots verbatim and restarts as its saved executable did: needs, contexts, the embedded assembler, the FFI layer and reva.f's own REPL run. Fixed: `;;` patched a byte inside the previous call's offset when alignment padding sat before the `ret`. A virtual libc (getenv, putenv, localtime, errno) and the Linux syscalls reva.f uses; the virtual install prefix /reva/. 54 of 132 examples run (was 0). DESIGN.md 3.6.
- 15:54 `docs` The old wiki deprecated; docs/ started from the portable core: getting-started, gotchas, parity (every wiki page, chapter and example area with status) and parity-examples (generated); every Forth sample is checked by rust/scripts/doc-check.sh (golden docs-samples), every example classified by parity-examples.sh.
- 15:48 `build` The legacy build's paths after the move (legacy/Makefile, legacy/tools/build.f); the moved cores assemble to identical sections for win32, elf and macho. legacy/README.md maps old to new paths.
- 15:48 `refactor` The deprecated x86 implementation moved under legacy/ (assembly cores, Makefile, prebuilt binaries, scratch files), renames only; the Forth system (src/reva.f, lib/, examples/, tests/, bench/) stays.
- 15:43 `docs` DEPRECATED.md: the 32-bit x86 implementation is deprecated, with the plan (deprecate, move under legacy/, delete once the portable core covers it).
- 15:43 `docs` README.md for the fork, the original README preserved, LICENSE-MIT for new portions and LICENSING.md saying which terms apply where (Reva Forth stays public domain).
- 15:43 `feat` The portable Reva core in Rust: vx86, an IA-32-subset VM over Reva's register roles and a flat 32-bit memory, with the newcore.asm routines as Rust gateways at fixed addresses and a resumable dispatcher; reva-cli (the `reva` command) and reva-web (a wasm32 module with no imports, a web console); src/reva.f boots verbatim up to its embedded assembler; unit tests and 19 goldens. rust/DESIGN.md.
