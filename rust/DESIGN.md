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
            │  reva-cli (terminal, files)    │   push input, pull output,
            │  reva-web (wasm32, JS glue)    │   run in budgeted slices
            └───────────────┬────────────────┘
                            │ Host trait, Status
┌───────────────────────────▼──────────────────────────────┐
│ reva-vm                                                    │
│  machine   dispatcher: step = one instruction or gateway   │
│  x86       decode (bytes → Insn, cached) + exec (Cpu)      │
│  gates     newcore.asm routines in Rust, at fixed addrs    │
│  mem       flat u32-addressed memory, code-page tracking   │
│  heap      allocate/free/resize                            │
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

`x86/decode.rs` covers the one-byte and `0F` opcodes that occur in Reva's
output and sources, plus their regular neighbours: all ALU ops in all
forms, shifts/rotates, `mul/imul/div/idiv`, `lods/stos/movs/scas/cmps` with
`rep`, near `jmp/call/ret/jcc/loop`, `setcc/cmovcc`, `movzx/movsx`, `66`
prefix. Flags are exact except AF. Anything else faults with the bytes and
the Forth word containing them. Not modelled, deliberately: segments,
privileged/system instructions, x87/MMX/SSE (x87 is next; see §7).

### 3.3 Gateways: the core in Rust

Each newcore routine is a `Gate` at `0x1000 + 4n`. Compiled code reaches
it by `call`/`jmp`/`call [vector]` exactly as before; the dispatcher runs
Rust instead of decoding bytes. Dictionary entries keep newcore's layout
`[class][link][xt][count][name]`, class handlers are gateways, DEFER
vectors are cells holding addresses — so `reva.f` code that manipulates
any of these keeps working.

Where an assembly routine fell through into the next (`cclass` into
`vclass`, `cold` into `bye`, `literal` into `comma`), the Rust keeps the
combined behaviour; comments in `gates.rs` name each case.

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
image (`--save-image`) is the portable form of that executable: it starts
the same way.

**Virtual install prefix.** The core reports its program path as
`/reva/bin/lin/reva`, the original layout, so `reva.f`'s `libdir` gives
`/reva/lib/` without changing a line. Hosts map `/reva/` to real files: the
CLI to `$REVA_HOME` or the checkout it was built from; the web host to
embedded files (next step).

**The OS layer (`ffi.rs`).** `reva.f` selects its Linux personality
(`os` = 1) and reaches the OS through `lib`/`func:` and `syscall`. The port
serves one virtual library, libc, whose functions (`getenv`, `putenv`,
`localtime`, `errno`) are gateways over `Host`; any other library or
function resolves to 0, as a failed `dlopen`/`dlsym` did, so `reva.f` names
what is missing. Linux i386 `syscall` numbers used outside file I/O (time,
`gettimeofday`, `nanosleep`, `getpid`, terminal `ioctl`) map to `Host`; the
rest return `-ENOSYS`. The VM itself never reads a clock or the environment,
so it stays deterministic and the browser host needs nothing.

## 4. Non-functional requirements

| Quality | How it is addressed |
|---|---|
| **Portability / availability** | Pure Rust, no `unsafe` in the VM, no dependencies; one core for CLI and `wasm32` (module has zero imports). |
| **Reliability** | All memory access bounds-checked; null guard; return-stack floor; data-stack underflow check between words; faults recover to the REPL and roll back a half-made definition (as newcore did). |
| **Traceability** | Every gateway names its newcore routine; deviations are commented; fault reports name the Forth word and offset (`in bad+0xc`); `--trace` logs each step. |
| **Maintainability / understandability** | Small modules with literate headers; the boot source is `reva.f` verbatim plus two documented lines (§3.6) — no fork of the Forth code. |
| **Testability** | Unit tests per module, API tests (`reva-vm/tests`), and a reg-rs golden-output suite (`work/reg-rs`) over the CLI and the wasm build. |
| **Serviceability** | `describe_addr`, `--trace`, `--show-boot`, the `probe` example (`see WORD` disassembles). |
| **Upgradability** | Images contain only vx86 bytes and are versioned; translations are caches, never persisted. |
| **Performance** | Interpreter with decode cache now; basic-block translation to x86-64 / ARM64 / wasm later (§6). |
| **Usability** | `reva` behaves like the original REPL (banner, `ok>`, `reva.f`'s error messages); piped input runs as a batch without prompts. |

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
   personality) and restarts as its executable did (§3.6). Next: file words
   (`open/r`, `read`, …) through `Host`, with a virtual file system for the
   browser (bundled read-only files plus browser storage); then `(save)` /
   `makeexe` writing images; loading images in the browser.
3. x87 subset (as a 8-deep `f64` stack) for `lib/math/floats`, with 80-bit
   memory operands (`fld`/`fstp tbyte`).
4. OS layer: ✅ virtual libc and a portable `syscall` subset (§3.6). Next:
   native libraries via `libloading` (CLI) and JS imports (web).
5. Translators: x86-64, then ARM64, then wasm.
