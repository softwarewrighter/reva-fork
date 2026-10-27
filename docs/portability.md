# Portability: what the portable core runs, and what it stops

Reva was a 32-bit x86 program that called Windows, Linux and macOS
libraries directly. The portable core runs Reva's own x86 code in a VM
(see [DESIGN.md](../rust/DESIGN.md)), so most of that still works unchanged.
This page lists what does not, and what you see when a program reaches it.
Every message below ends with `(see docs/portability.md)`, which points here.

## The rules

| Code that depends on | What happens | Why |
|---|---|---|
| 4-byte cells, little-endian memory, Reva's register roles (`inline{ }` bytes, `cells/` as `sar eax,2`) | **runs** | The VM keeps cells 32-bit on 64-bit hosts and in WebAssembly: a fixed design rule, so saved data and machine-code words stay valid |
| Integer x86 instructions that Reva and its libraries emit | **runs** | The VM decodes that subset (DESIGN.md §3.2) |
| x87 floating point | **runs**, with `f64` registers: 80-bit values round to double on load, so the last extended-precision bits can differ | DESIGN.md §3.10 |
| MMX, SSE, segment registers, I/O ports, privileged or system instructions | **stops** | no portable meaning, or not used by Reva's own code |
| A C library other than the virtual libc | **stops** at its first function, with a note naming the library | see "Native libraries" below |
| `shell` (C `system`) | **stops**, unless the CLI was started with `--allow-shell` | running OS commands is the user's choice, not the program's |
| Windows-only libraries (registry, COM, Windows CE RAPI, Win32 console and clipboard, SEH) | **stops**, like any native library | Windows APIs have no portable equivalent |

"Stops" means the current line is abandoned and the session goes on, as
after any error: the dictionary and the stacks are reset the way Reva's own
error handler does.

## Native libraries

The portable core loads **no native libraries**, on purpose. Forth passes
32-bit VM addresses as pointers, and native 64-bit or WebAssembly code cannot
use them, so a general `dlopen` would crash rather than work. Instead, C
libraries are added as *virtual libraries*: Rust code inside the VM that
answers to the old library and function names and converts the arguments.
The Forth sources, with their `lib` and `func:` lines, stay unchanged.

Served today: libc's `getenv`, `putenv`, `localtime`, `gmtime`, `opendir`,
`readdir`, `closedir` and `system`, plus `errno`; and SQLite for
`lib/db/sqlite` through *host calls*, which the host answers (the CLI and
the desktop app; the browser next; see
[sqlite.md](sqlite.md) and rust/DESIGN.md 3.11). Planned: a UI service in
place of IUP and GTK, then regex, zlib, iconv and HTTP as programs need
them. Every planned library is permissively
licensed ([licenses.md](licenses.md)).

A library that is not served resolves to 0, as a failed `dlopen` did, so
`reva.f` reports the function by name and throws (catchable, as before). The
first time a function is looked up in such a library, the VM adds a note
naming the library. On the CLI the note goes to stderr; in the browser it
appears in the output.

```text
" libnothere.so" lib nothere
1 func: nofunc
nofunc
```

prints

```text
note: native library libnothere.so is not available in the portable core (see docs/portability.md)

ERROR: could not load nofunc

Caught: -2
```

A `(call)` of an address that is not a virtual-library function stops with
`call to native code at 0x...`.

## Unsupported instructions

The fault names the instruction family, the bytes and the Forth word that
contains them:

```text
*** unsupported SSE instruction at 0x0014792c: 0f 58 c1 c3 00 00 00 00 (see docs/portability.md) (in sse+0x0, eip 0x0014792c)
```

The families named are x87 floating point, MMX, SSE, SSE2, segment
override, segment register, I/O port (privileged), privileged, software
interrupt, processor identification (cpuid/rdtsc), undefined instruction
(ud2), lock prefix, and plain "x86" for anything else outside the subset.
Assembling such code with the built-in assembler works; only running it
stops.

## `shell`

`lib/os/shell`'s `shell` calls C's `system`. The VM serves `system`, but
only a host that allows it runs the command:

- **CLI:** `reva --allow-shell` runs the command with `sh -c` and returns the
  wait status, as C's `system` does (`shell_code @ 256 /` is the exit code).
  Without the flag, `shell` stops with
  `shell (running an OS command) is not allowed here`.
- **Browser:** never.

`shell$` also needs `tempnam`, which the virtual libc does not serve yet.

## Checking a program before you run it

Search for what will stop: `lib "` lines name native libraries, and
`inline{ }` or `asm{ }` blocks with vector (MMX/SSE) instructions will
fault when they run. A static report tool is planned.
