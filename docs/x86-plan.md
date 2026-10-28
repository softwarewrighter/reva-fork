# Running Reva's x86: what works, what breaks, and the plan

Reva compiles Forth straight into **32-bit x86 machine code**, and much of
`src/reva.f` itself is written as raw instruction bytes. Today's 64-bit
operating systems and browsers will not run that code (macOS dropped
32-bit programs entirely; browsers run no x86 at all). reva-fork therefore
runs Reva's x86 on a CPU written in software: **vx86**, in
`rust/reva-vm`. This page explains how that works, what Reva programs can
expect, what breaks, and what is planned.

Background and details: [rust/DESIGN.md](../rust/DESIGN.md) (sections 3
and 6), [portability.md](portability.md), [parity.md](parity.md),
[parity-examples.md](parity-examples.md).

## In one paragraph

vx86 is an interpreter for the part of x86 that Reva uses. It holds
Reva's whole world in one 32-bit block of memory (a byte array; in the
browser, WebAssembly's memory), keeps eight registers and the flags in
plain variables, and runs one instruction at a time: decode the bytes at
the instruction pointer (remembered per address), do what they say. The
routines the original core wrote in assembly (the interpreter, `find`,
`header`, files ...) are rewritten in Rust and sit at the fixed addresses
Reva calls them at, so Reva's code cannot tell. Because the VM can pause
after any instruction, the same core runs in a terminal, a browser tab and
the desktop app.

## How it works

| Part | What it does | Where |
|---|---|---|
| Memory | one flat 32-bit address space: Reva's code, dictionary, stacks, heap | `reva-vm/src/mem.rs`, `layout.rs` |
| CPU | registers (Reva's roles: EAX top of stack, ESI data stack, ESP return stack), flags, EIP | `reva-vm/src/x86/` |
| Decoder | bytes to instructions, cached per address; a write into code (Reva patches `then`, tail calls, `does>`) drops the cached instructions of that 256-byte page | `x86/decode.rs`, DESIGN 3.5 |
| Gateways | the original core's assembly routines in Rust, at the addresses Reva expects; reaching one runs Rust instead of decoding | `gates.rs`, DESIGN 3.3 |
| Continuations | a Rust routine that calls back into Forth does so without the Rust stack, so the VM can stop anywhere (waiting for input, for a host call) and resume later | DESIGN 3.4 |
| x87 FPU | the floating-point unit, emulated | `x86/fpu.rs`, DESIGN 3.10 |
| OS layer | the Linux i386 system calls `reva.f` makes, mapped to the host; a virtual libc (`getenv`, `localtime`, `opendir` ...) | `ffi.rs`, `fileio.rs` |
| Virtual libraries | Rust in place of the C libraries Reva loaded with `lib`/`func:`: libc, `turtle`, `highlight`, `reva-sqlite` | `ffi.rs`, `sqlite.rs`, DESIGN 3.11 |

## Which x86 vx86 runs

**Integer instructions:** what occurs in Reva's output and sources, plus
their regular neighbours: every ALU operation in every form, shifts and
rotates, `mul`/`imul`/`div`/`idiv`, `lea`, `mov`, `movzx`/`movsx`,
`push`/`pop`, `xchg`, `bswap`, `cwde`/`cdq` and their 16-bit forms
`cbw`/`cwd`, near `call`/`ret`/`jmp`/`jcc`/`loop`, `setcc`/`cmovcc`, the
string instructions (`movs`, `stos`, `lods`, `scas`, `cmps`) with `rep`,
and the `66` operand-size prefix. Flags are exact except the adjust flag
(AF), which nothing in Reva reads.

**Floating point (x87):** the whole stack-based FPU: loads and stores in
every format (32-, 64- and 80-bit floats, exact on store; 16-, 32- and
64-bit integers; `fisttp`), arithmetic in all forms, compares (`fcom`,
`fucom`, `fcomi`, `ftst`, `fxam`), `fxch`, `ffree`, `fcmov`, the
constants, control and status words with rounding control, `finit`/
`fclex`, and the transcendentals (`fsin`, `fcos`, `fsincos`, `fptan`,
`fpatan`, `f2xm1`, `fyl2x`, `fyl2xp1`, `fscale`, `fxtract`, `fprem`,
`fprem1`, `frndint`, `fsqrt`). `lib/math/floats` passes its own tests.

One difference: values are held as 64-bit doubles, not the x87's 80-bit
extended format, so the last digits of a long calculation can differ from
the original ([gotchas.md](gotchas.md)). Not provided: saving and
restoring the whole FPU state (`fsave`/`frstor`/`fldenv`/`fstenv`) and
the BCD forms.

**Deliberately not modelled:** MMX, SSE and later vector extensions;
privileged and system instructions; I/O ports; segment registers. Each
stops the current line with a message naming the instruction family and
the Forth word it is in, and the session goes on.

Instructions are added when Reva code needs them: `bswap` (SHA-1), `mov`
with a 32-bit absolute address (`lib/util/locals`, which CoSy uses), and
`cbw`/`cwd` (the disassembler, below).

## What works

- **All of `src/reva.f`** boots, unchanged, and restarts the way its saved
  executable did: the interpreter and compiler, contexts, `needs` and the
  libraries in `lib/`, the embedded assembler, `catch`/`throw`, files,
  saved images, floats.
- **Plain Reva programs** run unchanged: **71 of the 132 programs in
  `examples/`**, on every host (CLI, browser, desktop app).
- **Hand-written machine code** (`inline{ ... }`, words built from bytes,
  the embedded assembler's output) runs as long as it stays within the
  subset above.
- **SQLite** (`needs db/sqlite`) on the CLI and in the desktop app, with
  the browser next ([sqlite.md](sqlite.md)).
- **Turtle graphics**, colour source listings (`pp`), REPL history, the
  live demo: new in the fork ([parity.md](parity.md)).

## Seeing the x86: `see`

In the original, `see WORD` shows the machine code a word compiled to:

- plain Reva: a **hex dump** of the word's bytes;
- after `needs debugger`: a **disassembly**, by `lib/util/disasm` (Reva's
  port of the Open Firmware 386 disassembler, itself written in Forth).
  CoSy loads the debugger, which is how it showed "the x86 under a
  definition" at once.

Because Reva's code is real x86, the listing in the fork shows exactly
what vx86 executes:

```text
needs debugger
: sq dup * ;
see sq
0014F098    lea      esi, [esi-4]      | 8D 76 FC     dup: make room
0014F09B    mov      [esi], eax        | 89 06        dup: copy the top item
0014F09D    mul      [esi]             | F7 26        *
0014F0A0    add      esi, 4            | 83 C6 04     *: drop the used item
0014F0A3    ret                        | C3
```

Calls name their targets (`call allocate`, `jmp fill` for a tail call).
Until saga step `local-see` it crashed: the disassembler's own code uses
`cbw` (`66 98`), which vx86 did not decode, and the crash left `base` in
hex. Now vx86 decodes `cbw`/`cwd`, and a core routine written in Rust
(its "code" is a gateway address) is reported as such instead of being
disassembled forever:

```text
see allocate
core routine, written in Rust (vx86 gateway at 00001124 )
```

`see` was run over every word in the booted dictionary (356): none
faults or runs away, and `base` is decimal afterwards (golden `see`).

## How legacy Reva programs use this, and what breaks

A program's Forth source is compiled by `reva.f` into x86, which vx86
runs. Anything that stays inside Reva works. What breaks is where the
original reached **outside** its own x86: to the operating system, to C
libraries, or to instructions vx86 leaves out. From the parity report
(132 examples: 71 run, 33 stop on a C library, 12 are interactive or never
end with stdin closed, 9 miss a word, 7 fault):

| What breaks | Why | Examples today | How to port it, and to what |
|---|---|---|---|
| **C libraries** through `lib`/`func:` | vx86 cannot call native code: a Forth pointer is a 32-bit VM address that native 64-bit or WebAssembly code cannot use | 33: iconv (23, the first library IUP-based GUI programs load), terminal window size (`window_xy`, 4), expat XML, pthreads, IMath bignums, GTK, gpgme | A **virtual library**: Rust inside the VM answering the same function names with integers and `( a n )` strings, never pointers, as `reva-sqlite` does. Rust crates cover most: `encoding_rs` (iconv), `quick-xml` (expat), `num-bigint` (IMath), `flate2` (zlib), `regex` (pcre). Threads become host tasks, if needed at all |
| **GUI toolkits**: IUP, GTK, OpenGL, Win32 windows | Native, platform-bound, and pointer-heavy | 29 GUI programs (most stop at iconv first) | The planned **UI service** (panes, keys, the turtle canvas, SVG, later WebGL) served by the browser page and the desktop app; selected demos ported to it. IUP/GTK code itself is not ported |
| **OS calls beyond the mapped set**: processes, signals, sockets, terminal control | Only the system calls `reva.f` itself needs are mapped | `dhttpd.f` (`tcpsock`) | **Host services**, as files and SQLite are: sockets and HTTP on the CLI first; in the browser through `fetch` where it makes sense |
| **Instructions outside the subset**: SSE/MMX in hand-written code; `asm{` blocks that ran the external FASM assembler | Not modelled, or a tool that is not there | `notready/galife.f` (`lib/asm`); no example uses SSE today | Rewrite in plain Forth or in integer/x87 code, or add the instructions to vx86 if several programs need them |
| **Code written for an older Reva** (7.0.8): `value` without an initial value, `0L println` | Language changes between Reva versions, not emulation | `sqlite.f`, `zipdb.f`, `db-report.f` | Small source edits; CoSy carries a compatibility file (`compat708.f`) |
| **Native executables** (`makeexe`, `save` as an executable) | There is no native binary to write | `dump.f`, `datediff.f` run, saving images | Saved **images** (`save`, `reva --image`) with the `reva` binary or the web build |
| **80-bit precision** | Doubles inside | rare | Documented; extended precision in software only if a program proves to need it |
| **Speed** | Interpreting each instruction costs far more than running it natively; not yet measured against the original | CPU-heavy loops | The planned **translators** (below) |
| **Interactive programs** in batch runs | They wait for keys; the parity run closes stdin after 5 s | 12 | Not a defect: they run interactively |

## The plan

In the order the local lane's saga (`.agentrail/plan.md`) takes them, with
what is not scheduled yet at the end.

1. **`see`** (step `local-see`): `cbw`/`cwd`, core routines named, golden.
   Done in this step.
2. **SQLite everywhere** (steps `local-sqlite*`): CLI and desktop done,
   browser in progress; then `help` through the original SQL-based
   `lib/helper`, with `help.db` generated from `src/help.txt`.
3. **Examples triage** (step `local-examples-triage`): the programs that
   fault or miss a word, unrelated to C libraries: fix vx86 or core gaps
   test-first, record example bugs.
4. **Benchmarks** (step `local-perf-baseline`): `just bench` on the CLI
   and in WebAssembly, against the original's numbers where `bench/` has
   them; the reference for the translators.
5. **Virtual libraries, by how many programs they unblock** (to schedule):
   iconv first (23 programs move past it), then terminal size, XML, big
   integers; each pointer-free and host-neutral, like `reva-sqlite`.
6. **The UI service** (to schedule): panes, keys, text, an SVG surface in
   the page and the desktop app; selected GUI demos ported to it.
7. **Host services** (to schedule): sockets and HTTP on the CLI.
8. **Translators** (DESIGN.md section 6, roadmap item 5): turn hot vx86
   code into native x86-64, ARM64 or WebAssembly at run time. vx86 stays
   the reference: a translation is a cache over it, dropped when the code
   it came from is written to. Later, `see` could show a word's
   translated code beside its x86.
9. **Instruction coverage on demand**: add x86 instructions when a real
   program needs them, each with a decoder test and an execution test;
   never SSE "just in case".

## Testing

- Decoder and executor unit tests for every instruction group
  (`x86/decode.rs`, `x86/exec.rs`, `x86/fpu.rs`).
- Machine tests through the public API (`reva-vm/tests/machine.rs`).
- Goldens (`rust/work/reg-rs`): boot, core words, errors, files, floats,
  images, SQLite, `see`, the parity report of all 132 examples, and every
  code sample in `docs/`.
- `just check` runs all of them before every commit.
