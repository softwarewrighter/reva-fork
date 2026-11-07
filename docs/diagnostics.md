# Diagnostics: finding the word at fault

When something goes wrong, reva-fork says where: the file and line, the
word, the stack, and the words that called the one at fault. Some checks
cost nothing and are always on. Others are opt-in, because they stop
programs that Reva let go on. They work the same way in every host: the
CLI, the browser and the desktop app.

| You want | Always on | Opt-in |
|---|---|---|
| where an error or fault happened | `FILE:LINE`, the word, the stack, the callers | |
| an underflow hidden inside a definition | a warning between words | `diagnose underflow-at-word` |
| an `execute` of what is not code | the fault after the jump | `diagnose execute` |
| what a word compiles, and who compiled it | | `diagnose compile` |
| a loop that never ends | the runaway guard ([gotchas.md](gotchas.md#a-runaway-loop-pauses)) | |
| Windows-only paths and commands | | `reva --port-check` ([porting-from-windows.md](porting-from-windows.md)) |

## Errors and faults say where

An error in an included file starts with the file and line, and the files
that included it, as a compiler's do:

```text
inner.f:3 (from outer.f:7): ERROR: unknown word: frobnicate
```

A fault (a bad address, a division by zero, an instruction the core does
not run) names the word it happened in, the words that called it (innermost
first, up to the word the interpreter ran, at most 8), the place in the
input and the stack:

```text
: a 0 @ ;
: b a 1 ;
: c b 2 ;
c
*** bad memory access: 4 byte(s) at 0x00000000 (in a+0xc <- b <- c, eip 0x00146ce8) after `c`
stack (1): 0
```

`a+0xc` is the offset into `a`'s code. The callers come from the return
stack, so a word whose last call is a jump (Reva compiles the call at the
end of a definition as a jump) leaves no trace: with `: d c ;`, `d` still
shows `a+0xc <- b <- c`. After a fault the REPL starts again, a half-made
definition is undone, and the stack is empty.

## `diagnose`: opt-in checks

`diagnose NAME` turns a check on, `diagnose off` turns them all off, and
`diagnose` alone lists them, with `*` before those that are on:

```forth
diagnose underflow-at-word
diagnose
diagnose off
```
```output
diagnose NAME (or off); * is on:
  * underflow-at-word    stop at the word that leaves the stack below empty, with its callers
    execute              stop an execute (any indirect call or jump) of what is not code
    compile              say what is compiled into a definition, and who compiled it
```

Each also has a CLI flag and a page option, so a program can be checked
from its start:

| Check | Forth (any host) | CLI | Web page |
|---|---|---|---|
| underflow at the word | `diagnose underflow-at-word` | `--underflow fail-at-word` | `?diagnose=underflow-at-word` |
| execute of what is not code | `diagnose execute` | `--check-execute` | `?diagnose=execute` |
| trace compiling | `diagnose compile` | `--trace-compile` | `?diagnose=compile` |

In the desktop app, type the Forth form (or put it first in a package's
start files). The checks cost no measurable time over the 133 programs in
`examples/`.

### Underflow inside a definition

The depth is checked between the words of a line. By default an underflow
there is a warning, once, and Forth goes on in the slack above the stack as
Reva did (`--underflow fail` stops instead); the next line typed at the
prompt starts from an empty stack. A word that takes too much inside a
definition, when a later push brings the depth back, is not seen: here the
`1` given to `f2` is gone, and no warning says so:

```forth
: f1 drop drop ;
: f2 f1 1 ;
1 f2 .s
```
```output
(0)
```

With `diagnose underflow-at-word` the depth is also checked whenever a word
returns, and the first word that returns with the stack below empty stops,
with its callers:

```text
*** stack underflow (in f1+0xa <- f2, eip 0x00146ce6) after `f2`
```

Among the examples it stops six programs: four already warn, and in two a
later push hid the underflow (`quoth.f`'s `choose` gives `seed4` two of its
four values).

### `execute` of what is not code

`execute` jumps to whatever is on the stack, so `0 execute` faults at
address 0 and the report cannot say who executed it:

```text
*** bad memory access: 1 byte(s) at 0x00000000 (in 0x00000000 <- run, eip 0x00000000) after `run`
```

With `diagnose execute`, every indirect call or jump (`execute`,
`@execute`, deferred words) must reach code: a core routine, or code
compiled into the dictionary. Otherwise it stops before the jump, in the
word that executed:

```text
*** execute: 0x00000000 is not code (not a word, a quotation or a core routine) (in go+0xd <- run, eip 0x00146ce9) after `run`
```

Code a program builds in allocated memory counts as not code, so such a
program needs the check off.

### What is compiled, and by whom

`diagnose compile` says each word compiled into a definition (`compile:`
for a call, `inline:` for copied bytes), and the words that compiled it when
it was not the interpreter. It shows a word that compiles where it should
run:

```text
: allocate prior allocate 1 + ;
macro : m 1000 allocate . ; forth
: t m ;
compile: core:Allocate into t by allocate <- m
1001
```

`m` is a macro, so it runs while `t` is compiled, and `prior` compiles the
old `allocate` by its class, which compiles while the compiler is on: core
`allocate` went into `t`, and `.` printed the size given back. Reva 7.0.8
does the same; see [gotchas.md](gotchas.md#prior-compiles-while-the-compiler-is-on)
for the fix.
