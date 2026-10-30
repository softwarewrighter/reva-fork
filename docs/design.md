# Portable Reva core — design

This document explains *why* the port looks the way it does. For *how to
use it*, see [README.md](README.md).

## 1. Problem

Reva is a 32-bit x86 Forth:

- the core (`src/newcore.asm`, ~2,000 lines, plus one ~550-line OS layer
  per platform) is NASM assembly;
- the compiler emits x86 machine code directly (`compile` writes
  `call rel32`, `literal` writes a 10-byte push sequence, `if` writes
  `test eax,eax; lodsd; jz rel32`, `;` turns a trailing call into a jump);
- about 60 words in `src/reva.f` and about 100 in the libraries are
  *written* as instruction bytes (`: + inline{ 01 06 } drop ;inline`).

32-bit x86 is going away (macOS dropped it in 10.15; 64-bit Linux needs
multilib), and none of it can run in a browser.

## 2. Options considered

| Option | Reuse of existing Forth | Native speed | Browser | Cost |
|---|---|---|---|---|
| Token-threaded Rust VM | low: every machine-code word, every control structure and the compiler must be rewritten | interpreter, 2–5× slower | yes | high |
| Hand-written x86-64 / ARM64 assembly cores | low (same rewrite, twice) | yes | no | very high |
| **vx86: run Reva's IA-32 subset in a Rust VM** (chosen) | **high: `reva.f` runs byte-for-byte** | interpreter now; native via translation (§6) | yes | medium |

The deciding observation: Reva's world is already a *flat 32-bit address
space with fixed register roles* — exactly what `wasm32` linear memory is,
and what a 64-bit host can model with one base register. Reva's compiled
code uses only a small, regular slice of IA-32. Executing that slice is
far less work than rewriting everything that produces it.

## 3. Architecture

```
            ┌──────────── hosts ────────────┐
            │  reva-cli (terminal, StdFs)    │   push input, pull output,
            │  reva-web (wasm32, MemFs, JS)  │   run in budgeted slices
            └───────────────┬────────────────┘
                            │ Host trait, Status
┌───────────────────────────▼──────────────────────────────┐
│ reva-vm                                                    │
│  machine   dispatcher: step = one instruction or gateway   │
│  x86       decode (bytes → Insn, cached) + exec (Cpu)      │
│  gates     newcore.asm routines in Rust, at fixed addrs    │
│  mem       flat u32-addressed memory, code-page tracking   │
│  heap      allocate/free/resize                            │
│  ffi       virtual libc, Linux syscall subset              │
│  fileio    file words + file syscalls over Host::fs        │
│  vfs       FileSystem trait, MemFs, embedded lib/          │
│  image     save/load (vx86 bytes only)                     │
│  translate hooks for native/wasm translators               │
└──────────────────────────────────────────────────────────┘
```

### 3.1 Memory and registers

One `Vec<u8>`; every address Forth sees is an offset into it
(`layout.rs` has the map, mirroring newcore's data section). Registers keep
Reva's roles: **EAX = top of stack, ESI = data-stack pointer, ESP = return
stack** (in VM memory, so `r>` on a return address works as it always did).
Addresses below 0x1000 fault, so `0 @` is reported, not silently read.

### 3.2 The instruction subset

The decoder (components/x86-decode) covers the one-byte and `0F` opcodes that occur in Reva's
output and sources, plus their regular neighbours: all ALU ops in all
forms, shifts/rotates, `mul/imul/div/idiv`, `lods/stos/movs/scas/cmps` with
`rep`, near `jmp/call/ret/jcc/loop`, `setcc/cmovcc`, `movzx/movsx`, `66`
prefix, and the x87 floating-point unit (§3.10). Flags are exact except AF.
Anything else faults with the bytes and the Forth word containing them. Not
modelled, deliberately: segments, privileged/system instructions, MMX/SSE.

### 3.3 Gateways: the core in Rust

Each newcore routine is a `Gate` at `0x1000 + 4n`. Compiled code reaches
it by `call`/`jmp`/`call [vector]` exactly as before; the dispatcher runs
Rust instead of decoding bytes. Dictionary entries keep newcore's layout
`[class][link][xt][count][name]`, class handlers are gateways, DEFER
vectors are cells holding addresses — so `reva.f` code that manipulates
any of these keeps working.

Where an assembly routine fell through into the next (`cclass` into
`vclass`, `cold` into `bye`, `literal` into `comma`), the Rust keeps the
combined behaviour; comments in the word families (components/words-*) name each case.

### 3.4 Never blocking: continuation gateways

A browser cannot block in `key`. So any routine that calls back into Forth
(interpret, query, eval, include, catch, `:` via the `(header)` vector) is
split at each call: the call becomes "push continuation gateway *k*, jump
to target", and the rest of the routine is *k*. No Forth state lives on the
Rust stack, so `run()` can return `NeedInput` (or `Yielded` when its budget
is spent) after any step and resume later. The CLI and the browser use the
same core.

Rarely-hooked callbacks (`do_cr`, `prompt`, `iterate`, user `emit`/`type`)
run as *nested calls* on the Rust stack; if one asks for input that is not
queued, a blocking host may supply it, otherwise it faults clearly.

### 3.5 Self-modifying code

Reva patches code (`then`, tail calls, `does>`). Decoded instructions are
cached per address; memory marks 256-byte pages that hold decoded code, and
a write into such a page queues it for invalidation (decode cache and any
translator). Writes to data pages cost one bit test.

### 3.6 Booting all of `reva.f`, and what stands in for its last step

`reva.f` is a build script: it compiles the whole system, then saves a new
native executable with `(save)` and exits. The saved executable, when run,
starts with fresh stacks, runs the `onstartup` list (which allocates
`scratch`, `pad` and the compiled-string area) and calls `appstart` (banner,
command line, then the `catch`-protected interpreter loop).

The port boots `reva.f` verbatim up to that last step, then does in place
what "save and run the executable" did (`embedded::boot_source`):

1. Before `reva.f` hides `(compstr)`, one inserted line hands the string area
   to `setextra`, exactly as `save` does, so the startup code finds the
   strings compiled during the boot.
2. Instead of `(save) bye`, the tail runs `(boot-restart)`: drop the boot
   file from `argv`, abandon the include frames, and jump to `cold`. It is
   also given `reva.f`'s interpreter loop (`exception-reset`), so fault
   recovery returns to the same loop rather than the core's bare REPL.

Why not implement `(save)` and run the result? A browser cannot exec a file,
and the image would be identical to the memory we already have. A saved
image is the portable form of that executable: it starts the same way, and
`(save)` writes one (§3.9).

**Virtual install prefix.** The core reports its program path as
`/reva/bin/lin/reva`, the original layout, so `reva.f`'s `libdir` gives
`/reva/lib/` without changing a line. Hosts map `/reva/` to real files: the
CLI to `$REVA_HOME` or the checkout it was built from; every host also sees
the copy of `lib/` compiled into the core (§3.7).

**The OS layer (`ffi.rs`).** `reva.f` selects its Linux personality
(`os` = 1) and reaches the OS through `lib`/`func:` and `syscall`. The port
serves one virtual library, libc, whose functions (`getenv`, `putenv`,
`localtime`, `errno`) are gateways over `Host`; any other library or
function resolves to 0, as a failed `dlopen`/`dlsym` did, so `reva.f` names
what is missing, and the VM adds a one-line note naming the library
(`Host::diag`; stderr on the CLI). `system` is served but runs only where
the host allows it (`Host::system`: the CLI with `--allow-shell`, never the
browser); otherwise it faults with how to allow it. Linux i386 `syscall` numbers used outside file I/O (time,
`gettimeofday`, `nanosleep`, `getpid`, terminal `ioctl`) map to `Host`; the
rest return `-ENOSYS`. The VM itself never reads a clock or the environment,
so it stays deterministic and the browser host needs nothing.

### 3.7 Files

`reva.f`'s file words bottom out in newcore's `open/r`, `open/rw`, `creat`,
`close`, `read`, `write`, `fsize`, and in Linux system calls (`lseek`,
`rename`, `unlink`, `stat`, `chdir`, `getcwd`; `mkdir`/`rmdir` and
`opendir`/`readdir` in `lib/os/dir`). The port keeps both layers as they
are and implements the bottom one (`fileio.rs`) over a small trait,
`vfs::FileSystem` (open/close/read/write/seek/stat/remove/rename/mkdir/
rmdir/list/chdir/cwd), that a host lends through `Host::fs`:

- **CLI:** `StdFs`, the real file system; `/reva/` maps to the install
  root. Descriptors are the host's own numbers (from 3), not OS fds.
  VM paths are always `/`-rooted and `/`-separated; on Windows the host
  path `C:\dir` is the VM path `/C:/dir` and back (`vm_path`/`host_path`
  in `reva-cli/src/fs.rs`), so one `vfs::normalize` serves every host.
- **Browser and tests:** `MemFs`, an in-memory tree. It records which
  paths changed, and `reva-web` exports them so `reva.js` can keep them in
  `localStorage` and put them back before the next boot. A page bundles
  its own files the same way (`Reva.load(url, { files, storage })`).

Results follow the kernel: a value or `-errno`, and `ioerr` is 0 or the
negative result, exactly as newcore's `call_linux_return` left it, so
library code that tests `ioerr` behaves as before. Descriptors 0–2 stay
the console (writing to 1 or 2 is output).

**`lib/` is embedded** (`build.rs`, feature `embed-lib`, on by default) and
appears read-only at `/reva/lib/` under both file systems: `needs` works in
the browser with no fetches, and an installed CLI binary needs no
checkout. On the CLI a file on disk wins over the embedded copy; in
`MemFs` the first write copies an embedded file. Cost: about 0.7 MB in the
binary and the `.wasm` (text, so it compresses well when served gzipped);
a host that does not want it builds without the feature.

Why a trait rather than more `Host` methods? The operations come as a set
with shared state (open descriptors, working directory), and the same
`MemFs` serves the browser and every test; the trait keeps that state out
of the VM, which stays deterministic and image-safe (open files and
directory streams are host state, not part of an image).

### 3.8 The browser: a worker, time slices, interrupt

The page must never wait for the VM. Booting `reva.f` is one call of
140 to 300 ms that cannot be sliced, and a step budget is not a time
budget (200k steps is 8 ms on a desktop, several times that on a phone).
So the browser host runs the VM in a **Web Worker** (`www/reva-worker.js`)
and the page only draws and stores (`www/reva.js` `start()`):

- **Time slices.** `Reva.run` runs 50k-step chunks until about 8 ms have
  passed, posts the output, and yields. Yields use a `MessageChannel`
  message (no 4 ms timer clamp), but at least every 25 ms a `setTimeout`,
  because a port's messages can starve other tasks (Node does this, and
  `scheduler.yield()` does it by design). That fair turn is what lets
  `input` and `stop` messages in while the VM computes. Measured: about 90%
  of the unsliced speed.
- **Messages:** `init`, `input`, `close`, `stop` to the worker; `output`,
  `status`, `files`, `error` back (table in `reva-worker.js`). Workers
  cannot see `localStorage`, so changed files travel to the page, which
  saves them whenever the VM waits for input or halts.
- **Drawing:** the page appends output at most once per animation frame and
  caps the console's length.
- **Input** never blocks a thread: `key` returns `NEED_INPUT` and the
  worker is idle until the next `input` message.
- **Interrupt:** `Machine::interrupt` (`reva_interrupt`) is taken at the
  next step of `run`: queued input is dropped and the VM recovers as from a
  fault (`*** interrupted (in WORD)`), keeping the dictionary. Code inside
  a nested call runs to completion within one step, so if the worker does
  not answer in 2 s the page terminates it and starts a fresh one.
- **Clock:** the module has no imports, so the page passes `Date.now()` in
  (`reva_set_time`) before each slice; tests keep the fixed clock, and the
  smoke test checks that 1k- and 1M-step slices print the same transcript.
- **Fallback:** where a worker cannot start, the same loop runs on the
  page's thread, using `scheduler.yield()` to let the page draw.

No `SharedArrayBuffer` or `Atomics.wait` (they need COOP/COEP headers that
a plain static server does not send), so any static directory serves it.

### 3.9 Saved images and `restart`

Reva turned a program into an executable with `save NAME` (`(save)` under
it) and relaunched itself with `restart`. Both are host matters here.

- **Images.** `(save)` ( a n -- ) writes the machine's state as an image
  (`image.rs`: statics, dictionary, code, heap; vx86 bytes only) to the
  named file through `Host::fs`, with `ioerr` as for the file words.
  `reva.f`'s `save` wrapper runs unchanged around it (it hands over the
  compiled-string area and prints the sizes). `save` may run from a file
  being included or inside a word; loading an image therefore resets the
  interpreter's source, `>in`, compile state and catch frame, as a freshly
  started executable had them, and starts at `cold`. `reva --save-image`
  writes the same format from outside.
- **Starting from one.** `reva --image FILE`; in the browser
  `start(wasm, { image })` (a URL or bytes; the worker fetches it), or
  `reva_boot_image` in the module's C ABI. An image skips compiling
  `reva.f` and whatever was loaded before saving, which is what a
  page needs to start a large program at once.
- **`restart`** is a core word (gate appended after the libc gates, so
  image addresses stay stable). It runs the `onexit` list as `bye` does,
  then the machine stops with `Status::Restart`. The host keeps a
  `reva_vm::Start`, the way it made the machine (boot with `argv`, or
  image bytes), makes a new machine from it, and hands over the input not
  yet read (`Machine::take_input`; the rest of the current line is gone,
  as it was in the old process). What the host owns survives: files, and
  the console. The CLI then includes its files and evaluates its `-e`
  texts again; the browser module does all of this inside `reva_run`, so
  JavaScript sees only output (a new banner). This replaces wrapper loops
  that relaunch on a special exit status.
- **Why the host and not the VM.** A restart is "run the program again"; a
  VM that reset itself would have to know how it was made (boot file,
  image, arguments) and would mix host policy into the core. One `Start`
  type keeps it identical on every host.

A script that calls `restart` unconditionally restarts for ever, as it did
on the original (and as a launcher loop would); stop it with Ctrl-C.

### 3.10 Floating point: the x87

`lib/math/floats` is x87 code written as bytes (`f+` is `inline{ de c1 }`,
`f@` is `fld tbyte [eax]`), and CoSy stores its floats in the x87's 80-bit
format. So the VM decodes the x87 (`D8`–`DF`, `fwait` as a no-op;
components/x86-exec's reva-x86-fpuexec) and keeps its state in the CPU: eight registers as a stack
with `TOP` and tags, the control word and the status word.

- **What is there:** loads and stores of every memory format (32/64/80-bit
  floats, 16/32/64-bit integers, `fisttp`), constants, the four
  operations in all their forms, compares (`fcom`, `fucom`, `fcomi`,
  `ftst`, `fxam`, condition codes `C0`–`C3`), `fxch`, `ffree`, `fcmov`,
  `fldcw`/`fstcw`/`fstsw`, `finit`/`fclex`, and the transcendentals
  (`fsin`, `fcos`, `fsincos`, `fptan`, `fpatan`, `f2xm1`, `fyl2x`,
  `fyl2xp1`, `fscale`, `fxtract`, `fprem`, `fprem1`, `frndint`, `fsqrt`).
  Not there: `fldenv`/`fstenv`/`fsave`/`frstor` and the BCD forms.
- **Registers are `f64`.** A software 80-bit float would be exact but slow
  and large, and every host has a fast IEEE double. 80-bit memory values
  convert exactly to the 80-bit format and round to nearest-even on the
  way in; 64-bit integers round like doubles. Arithmetic rounds to double
  whatever the precision field says, and the rounding field applies to
  `fist`/`frndint` only. Results can differ from a real x87 in the last
  bits of the extended format, which shows when a program prints more
  digits than a double holds, or truncates (`f.` prints `0.49999` for
  `sin 30°`).
- **Exceptions are masked**, as Reva left them: an invalid operation gives
  the indefinite NaN and sets `IE`, an empty register reads as a stack
  fault, a push onto a full stack overflows to NaN, and `fwait` never
  traps. The sticky flags are visible through `fstsw`.
- **No new imports:** the WebAssembly module still imports nothing; the
  `f64` functions come from Rust's standard library.
- The FPU state is part of the CPU, not of an image: a saved program starts
  with a fresh unit, as a new process did (`finit` state).

### 3.11 Host calls and SQLite

Decided with the user on 2026-10-26 (saga steps `local-sqlite-design`,
`local-sqlite`):

- **SQLite on every host.** The CLI and the desktop app use SQLite
  compiled into the native binary; the browser uses SQLite's official
  WebAssembly build ([sqlite.org/wasm](https://sqlite.org/wasm)) loaded by
  the page, with databases kept in the browser's private file system
  (OPFS). `lib/db/sqlite` programs run unchanged on all three.
- **No C calls in our code.** Native hosts use the `reva-sqlite` crate
  (components/sqlite), which drives SQLite through **rusqlite's safe API**;
  `libsqlite3-sys` (rusqlite's own dependency) compiles SQLite itself. No
  `unsafe`, no FFI of ours. `reva-vm` and `reva-web` gain no dependency.
  The workspace's rust-version is 1.89 (rusqlite 0.41).
- **No pointers in the Forth binding.** The old `lib/db/sqlite` called
  SQLite's C API directly: out-parameters (`sqlite3_open` writing the
  handle through a pointer), returned `char*`, and an x86 `callback:`
  thunk for `sqlite3_exec`. Nothing else used that layer: programs and
  libraries use the `~db` words (`sql_open`, `sql_exec`, `sql_prepare`,
  `sql_pexec`, `sql_pcol_text`, `sql_fetch` ...). So the library now sits
  on the virtual library `reva-sqlite`, whose functions (`rsql_open`,
  `rsql_step`, ...) take handles as small integers and strings as
  `( a n )`; a text answer comes back as its length, with the bytes in a
  VM buffer (`rsql_ptr`; one per statement and column, so a row's columns
  stay valid together). `sql_fetch` walks rows with `rsql_next` instead
  of a C callback, calling the Forth word with a stack of its own as the
  C callback did. Every public word keeps its stack effect.
- **Statements without lifetimes.** rusqlite's `Statement` borrows its
  connection, which does not fit handles kept between Forth calls. A
  `reva-sqlite` statement keeps its SQL and bindings; its first `step`
  runs it (rusqlite, safe API) and buffers the rows, later steps hand
  them out. A SELECT's result is held in memory until reset or finalize,
  which suits REPL-sized data.
- **One mechanism, the host call** (`reva_vm::hostcall`). A
  virtual-library function the VM cannot serve becomes a `HostCall`
  (service, operation, integers, byte strings: never VM addresses) for
  `Host::host_call`. The host answers at once (the CLI), later (the
  desktop's and the browser's worker, which must ask the page), or not at
  all (a clear stop, as for a native library the core does not serve).
  "Later" stops the VM with `Status::HostCall`; the host reads
  `Machine::pending_host_call`, answers with `Machine::resume`, and runs
  again. Nothing is popped from the data stack until the answer is in,
  so the call simply retries, as a gateway waiting for input does.
  - CLI: `reva-sqlite` at once; database names map to host paths as the
    file words map them.
  - Desktop: the worker posts the request to the page, the page calls a
    Tauri command, the native side answers from `reva-sqlite` (paths
    under the app's folder only), the worker resumes the VM.
  - Browser: the VM's worker answers itself (`sqlite.js`) with SQLite's
    official wasm build, loaded from jsDelivr at run time (pinned, not
    redistributed). Each database runs in memory; opening loads its OPFS
    file (`sqlite3_deserialize`) and every committed change writes it back
    (`sqlite3_js_db_export`, a writable stream). SQLite's own OPFS VFSes
    were tried and rejected: `opfs` needs cross-origin-isolation headers
    GitHub Pages cannot send, and `opfs-sahpool` holds its files
    exclusively, so a page kept in the back/forward cache, or a second tab,
    blocks the next one (seen in Chrome: the reloaded page silently fell
    back to memory).

  The same path serves later native services. Gates for the new
  functions are appended (3.9: images keep gate addresses).

### 3.12 Crates

Decided with the user on 2026-10-27 (saga step `local-crate-layout`): the
core meets sw-checklist's structure limits in full (at most 7 modules per
crate, 7 functions per module, 50 lines per function, 500 per file), with
the bench guard (`just bench-guard`) and every golden unchanged at each
move.

Rust allows `impl Machine` only in the crate that defines `Machine`, and
today eight modules add methods to it. So the split puts `Machine` low and
lets higher crates extend it:

| Crate | Holds | State |
|---|---|---|
| `reva-x86` | the CPU types, decoder, executor, x87, memory, faults, the memory layout | done (pure moves) |
| `reva-host` | the `Host` trait and `BufferHost`, host calls, the virtual file system, embedded sources (`reva.f`, `lib/`), version, encodings; owns the build script | done (pure moves) |
| `reva-core` | `Machine` (state, statics, stacks, heap, images) and the run loop, which dispatches each gate through a table of handler functions instead of one large `match` | next |
| `reva-words` | the gate handlers by family (interpreter, compiler and dictionary, memory and strings, numbers, files, OS, images, version), as functions over `&mut Machine` | done: the word components (3.13) |
| `reva-vlib` | the virtual libraries (libc, iconv, SQLite, turtle and canvas, highlight), the same way | later |
| `reva-vm` | builds the handler table, `Start`, translation hooks, and the public API: every old path is re-exported, so hosts do not change | throughout |

Dispatch through a table of function pointers replaces the `match` on
the hot path; the bench guard decides whether that stays.

### 3.13 Components

Decided with the user on 2026-10-28 (saga step `local-components`), after
sw-mlpl's layout: each part of the core is a *component*, its own Cargo
workspace under `components/<name>/` with its crates in `crates/`, and
the structure rules of [code-metrics.md](code-metrics.md) apply (at most
4 modules per crate and 4 crates per component; when a part is full, a
new module, crate or component, never a merge).

| Component | Crates | Depends on |
|---|---|---|
| `macros` | reva-macros: `machine_trait!`, `words!`, `ext_trait!` | — |
| `x86-types` | reva-x86-base (faults, the address map), -mem, -fpustate (the x87 registers), -types (`Op`, `Insn`, `Cpu`) | — |
| `x86-decode` | reva-x86-cursor, -opcodes (one decoder per opcode group), -fpudec (the x87's), -decode (prefixes, a 256-entry opcode table, `0F`) | x86-types |
| `x86-alu` | reva-x86-ops (operands, ALU, stack, mul/div), -shift (shifts, rotates, strings), -float (pure: the 80-bit format, scaling, comparison) | x86-types, macros |
| `x86-exec` | reva-x86-fpuexec, -exec (`Exec::exec`, by instruction group) | x86-types, x86-alu |
| `x86` | reva-x86: a facade, every old path re-exported, with the traits that give `Cpu` its methods | the x86 components |
| `host` | reva-host | — |
| `core-parts` | reva-gate (the gateway table), reva-heap, reva-translate (translator hooks, the decode cache) | x86 |
| `canvas` | reva-canvas (the turtle and its display list), reva-canvas-out (op stream, SVG) | macros |
| `core` | reva-core (the `Machine` state and the stack and statics helpers every gateway uses), reva-core-ffi (`(call)` arguments, registers), reva-core-inspect (dictionary, state, settings) | x86, host, core-parts, canvas |
| `core-run` | reva-core-run (the run loop, nested calls, faults, input, host calls), reva-core-boot (new machines: statics and the core dictionary as tables; images) | core |
| `vlib` | reva-vlib | x86, host, core |
| `words-base` | reva-words-code, -text, -calls, -scan: the words' helpers (code space, pure number and string functions, calls into Forth, parsing) | core |
| `words-interp` | reva-words-classes, -interp, -eval: classes, default vectors, the outer interpreter, eval/include, catch, defining words | core, words-base |
| `words-dict` | reva-words-compile, -memory, -system: compiling and number words, memory and strings, bye/restart/version | core, words-base |
| `words-os` | reva-words-sys, -files, -libc, -ffi: system calls, file words, the virtual libc, `(lib)`/`(func)`/`(call)` | core, vlib |
| `words` | reva-words: the dispatch table (every gate to its handler) | the four above |
| `vm` | reva-vm | x86, host, core, vlib, words |
| `sqlite` | reva-sqlite | (rusqlite) |
| `cli` | reva-cli | vm, sqlite |
| `web` | reva-web | vm |
| `serve` | reva-serve | (web's www/, embedded at build time) |
| `desktop` | reva-desktop | sqlite (and web's www/, copied by build.sh) |

`Machine` is defined in reva-core and keeps only the helpers every
gateway uses; the rest of its methods are extension traits in the crates
above it (`machine_trait!`), which hosts bring in with
`use reva_vm::prelude::*`. The core dictionary and the statics' first
values are `const` tables.

`Cpu` is defined in reva-x86-types, and Rust allows `impl Cpu` only
there, so the executor's crates add its methods through extension traits
(`reva_macros::ext_trait!`, the pattern of `machine_trait!`); a caller
imports the trait (`reva_x86::Exec`). The split was checked against the
old one-crate executor: 9 million random decodes and 1.3 million random
executed instructions gave the same results, CPU state and memory.

The words are gate handlers, `fn(&mut Machine, Gate, &mut dyn Host)`,
in families; each family module lists the gates it serves
(`reva_macros::words!`), and reva-words assembles the lists into one
table indexed by gate, built at compile time (a gate listed twice or not
at all fails the build). The old 700-line `match` is gone; the bench
guard showed no change.

Components reach each other through path dependencies. They share the
repository's `target/` (`.cargo/config.toml`), one version (checked by
`scripts/check.sh`), one release profile (repeated in each workspace),
and one license policy (`deny.toml`). The gate runs fmt, clippy and the
tests in each component. The steps after this one take each component
to the code-metrics targets.

## 4. Non-functional requirements

| Quality | How it is addressed |
|---|---|
| **Portability / availability** | Pure Rust, no `unsafe` in the VM, no dependencies; one core for CLI and `wasm32` (module has zero imports). |
| **Reliability** | All memory access bounds-checked; null guard; return-stack floor; data-stack underflow check between words; faults recover to the REPL and roll back a half-made definition (as newcore did). |
| **Traceability** | Every gateway names its newcore routine; deviations are commented; fault reports name the Forth word and offset (`in bad+0xc`); `--trace` logs each step. |
| **Maintainability / understandability** | Small modules with literate headers; the boot source is `reva.f` verbatim plus two documented lines (§3.6) — no fork of the Forth code. |
| **Testability** | Unit tests per module, API tests (`components/vm/crates/reva-vm/tests`), and a reg-rs golden-output suite (`work/reg-rs`) over the CLI and the wasm build. |
| **Serviceability** | `describe_addr`, `--trace`, `--show-boot`, the `probe` example (`see WORD` disassembles). |
| **Upgradability** | Images contain only vx86 bytes and are versioned; translations are caches, never persisted. New gates are appended, so images keep working across core versions with the same image version. |
| **Startup time (browser)** | A page can start from an image (§3.9) instead of compiling `reva.f` and its program at every load. |
| **Performance** | Interpreter with decode cache now; basic-block translation to x86-64 / ARM64 / wasm later (§6). |
| **Usability** | `reva` behaves like the original REPL (banner, `ok>`, `reva.f`'s error messages); piped input runs as a batch without prompts. |
| **Responsiveness (browser)** | The VM runs in a Web Worker; the page draws once per frame. Measured in headless Chromium: the longest frame gap during a 15 s computation is 17 ms (on the page's thread before: 50 ms, and boot froze it for 133 ms). Stop takes effect within one 8 ms slice (§3.8). |
| **Data durability (browser)** | Files Forth code writes are saved to `localStorage` after every run slice that ends waiting for input or halted; a failed save (quota) is logged, not fatal. |

## 5. Deliberate deviations from newcore

1. End of input with a partial line interprets that line before exiting
   (newcore lost it).
2. Data-stack underflow is reported between words (newcore ran on in its
   underflow area).
3. Running out of code or dictionary space faults instead of exiting via
   `os_bye`, unless Forth code has redefined `heapgone`/`dictgone`.
4. `free` of a non-allocated address is ignored instead of crashing.
5. `fnvhash` of an empty string returns the offset basis (newcore looped
   2³² times).
6. `;;` does not turn a "call" into a tail jump when the five bytes before
   the `ret` end in alignment padding. newcore patched whatever byte sat at
   `here-5`; that was harmless on the x86 build only because, at its
   addresses, the byte was never `E8`. Here `reva.f`'s `::` (a `:` executed
   inside a definition, which pads with `nop`s) hit an `E8` inside the
   previous call's offset and corrupted it. This was the bug that stopped
   the boot at the assembler.
7. `localtime` is UTC: the port has no time-zone database.
8. Opening a directory with `open/r` fails with `EISDIR` (Linux opened it
   and failed on `read`). `read` from descriptor 0 fails with `EBADF`:
   keyboard input reaches Forth only through `key`, which can pause the VM.
9. `stat` fills the i386 `struct stat` and repeats `st_mtime` in
   `st_ctime`: `reva.f`'s `mtime` reads the ctime cell (offset 48).

## 6. The road to native speed

The interpreter defines semantics; translators are caches over it
(`translate.rs`). A translator receives a decoded basic block and returns
native code or declines; code writes invalidate translations by page.

- **x86-64:** nearly 1:1 re-encoding. VM registers → host 32-bit
  registers, memory operands gain a base (`[r15 + rax]`), ESP → a pinned
  register holding the VM return-stack offset, gateways exit to the
  dispatcher.
- **ARM64:** 1–3 instructions per vx86 instruction; `w`-registers for VM
  registers, `x28` memory base; flags via NZCV (mind the inverted carry
  after subtraction); icache flush and `MAP_JIT` on macOS.
- **WebAssembly:** a block (or a hot word) becomes a function over memory
  and a register array; the browser's JIT finishes the job.

Expected order of magnitude: interpreter ≈ 30–100× slower than native;
block translation should land within ~2× of the original x86 Reva.
(Estimates; `bench/` will replace them with numbers.)

## 7. Roadmap

1. ✅ vx86 core, gateways, resumable dispatcher, CLI, wasm host, `reva.f`
   prefix boots (through `does>`/`THROW:`).
2. ✅ All of `reva.f` boots (assembler, contexts, `needs`, FFI layer, OS
   personality) and restarts as its executable did (§3.6). ✅ File words
   and file system calls through `Host::fs`, with an in-memory file system
   for the browser (bundled files plus `localStorage`) and `lib/` embedded
   (§3.7). ✅ The browser host runs the VM in a Web Worker with time slices
   and Stop (§3.8). ✅ `save`/`(save)` write images, which start on the
   CLI and in the browser; `restart` (§3.9). `makeexe` (a native
   executable) is not planned: an image plus the `reva` binary or the web
   build is the portable form.
3. ✅ x87 (§3.10): an 8-deep `f64` stack with tags, control and status
   words, 80-bit memory operands, and the transcendentals;
   `lib/math/floats` passes its own tests.
4. OS layer: ✅ virtual libc and a portable `syscall` subset (§3.6); ✅
   clear stops for native libraries, `shell` (CLI `--allow-shell`) and
   unsupported instruction families ([portability.md](../docs/portability.md)).
   No generic native FFI: Forth passes 32-bit VM pointers, so C libraries
   come as virtual libraries (Rust bindings in the VM). Next: SQLite, then a
   UI service for the browser and the desktop app.
5. Translators: x86-64, then ARM64, then wasm.
