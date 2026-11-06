# Gotchas

Rebuilt from the old wiki's `gotchas.wiki`; every sample runs on the
portable core. Items that depend on parts not yet ported are listed at the
end.

## `'` vs `[']`

Both give the *xt* (execution token) of a word, but `'` parses the next
word *when it runs*. Inside a definition that means: when the definition is
executed, not when it is compiled. Use `[']` inside definitions to get the xt
of the word that follows in the source.

```forth
: sq dup * ;
: sq-xt ['] sq ;
4 sq-xt catch . .
```
```output
0 16
```

(`catch` runs an xt and leaves 0 when nothing was thrown.) With `'`, the word
is taken from the input when `later` runs:

```forth
: sq dup * ;
: later ' ;
later sq 5 swap catch . .
```
```output
0 25
```

## Character constants

`'A` is not a word: it is the number 65, the character code of `A`. So
`'(` and `'"` are character constants, not functions.

```forth
'A . '( emit '" emit cr
```
```output
65 ("
```

## Singles and doubles are integers

A trailing `L` makes a double-cell integer; it is pushed low cell first, so
the high cell is on top. Neither is a narrower kind of float.

```forth
1234L . .
-1L . .
```
```output
0 1234 -1 -1
```

## Number range

Numbers are 32-bit cells. Literals are checked for overflow, so
`-2147483648` (the most negative cell) cannot be typed directly; compute it.

```forth
2147483647 .
-2147483647 1- .
2147483647 1+ .
```
```output
2147483647 -2147483648 -2147483648
```

## An error in an included file stops that file

Typed at the prompt, an unknown word only skips the rest of its line. In a
file read with `include` (or given on the command line), the first unknown
word abandons the *rest of the file*, and the stack is restored to what it
was before the `include`. Check the first `ERROR:` line in the output: it
starts with the file and line, and the files that included it, as a
compiler's do (`inner.f:3 (from outer.f:7): ERROR: unknown word: ...`).
A fault says where too, and shows the stack:

```text
*** bad memory access: 4 byte(s) at 0x00000000 (in bad+0xc, eip ...) at outer.f:7 after `bad`
stack (4): 2 7 8 0
```

A file that is not there is reported (`include: nosuch.f: no such file`)
and `ioerr` is 1; the original Reva said nothing.

## Stack underflow is a warning

A word that takes more than the stack holds (`drop` on an empty stack)
prints ``warning: stack underflow (depth -1) at FILE:LINE after `drop` `` once, and Forth goes on, as
Reva did: words run in the slack above the stack, and later pushes bring it
back. Programs written for Reva rely on it (CoSy's Tui.f, and
`examples/sqlite.f`). `reva --underflow fail` stops with an error instead.

## What lies under an empty stack

Reva keeps the top of the stack in a register (EAX). With the stack empty,
that register still holds a leftover, the last value that was on top, and a
word that reaches one item too deep inside a definition (no warning: the
depth is checked between words) gets that leftover. `reset` sets it to 0, as
in every Reva: at each start of a saved program, so at every image start,
and after each error at the prompt. Code that frees or follows the leftover
then fails at a small address: CoSy's Tui.f `go` (its `>text` frees one
item below the string) worked after loading CoSy from source, but faulted
when started from an image or after an error, on Reva as here. Give the
leftover a harmless value first (for CoSy: `nil swap drop`), or keep the
item on the stack.

## Times are UTC

`time&date` and `localtime` report UTC: the portable core has no time-zone
database. The clock comes from the host (in the browser, the page passes
`Date.now()` in before each slice).

## Files: `ioerr` is `-errno`, and the browser keeps files in memory

The file words keep the original's results: after `open/r`, `read`,
`write`, `close`, `delete`, `rename`, `chdir`, ... `ioerr` is 0 on
success and the negative Linux `errno` on failure (`-2` for a missing
file), and a failed `open/r` returns that same negative number as the
"handle". Test `ioerr @`, not the handle.

Quirks of `reva.f` itself (unchanged, as on Linux x86):

- `rename ( a1 n1 a2 n2 -- )` removes one more cell than it takes (its
  system call reads three arguments). Keep a spare cell under the names.
- `seek ( offs h -- )` leaves nothing; use `tell` for the position.
- `creat` opens for writing only; reopen with `open/r` or `open/rw` to read.

`lib/` is built into the core and appears at `/reva/lib/` (read-only)
even without the files on disk, so `needs` works everywhere. In the
browser all files are in memory: the page bundles its own, and what Forth
code writes, or you upload, is kept in IndexedDB (the browser's quota for
the site; older pages used localStorage's 5 MB, moved over at first start). Empty directories are not kept, and modification times there
read as 0.

## Stop in the browser

**Stop** (Ctrl-C) takes effect within one slice, about 8 ms, and acts like
a fault: the rest of the input is dropped and a half-made definition is
removed. Code running inside a host callback (a redefined `emit`, `type` or
`cr`) cannot be interrupted that way; after 2 seconds the page replaces the
worker with a fresh one. Saved files survive that, but the session's
definitions are lost.

## `ms` waits everywhere

`1000 ms` waits a second in the browser and the desktop app too, as on the
CLI: the VM's worker waits and the page goes on drawing, so an animation
such as `: trees 13 2 do i tree-demo 1000 ms loop ; trees` shows one tree a
second. Stop ends a wait at once, and a wait is not running time for the
runaway guard below.

## A runaway loop pauses

A word that never ends, like `: spin repeat again ; spin` (or a word whose
last call is itself, which Reva compiles as a jump), would keep a CPU core
busy. In the browser and the desktop app, after 5 seconds of running
without waiting for input, the VM pauses: it uses no CPU, and a bar offers
**Stop**, **5 s more** or **Keep going**. A hidden tab pauses it as well,
and it goes on when the tab is shown again (`?background` in the URL keeps
it running). The CLI stops a run with `*** time limit` and exit status 124
after `reva --max-cpu SECONDS`. That limit defaults to 30 seconds with
`--ui-script` and is off otherwise (Ctrl-C stops a run at a terminal).

## `save` writes an image, not an executable

`save NAME` (and `(save)`, which takes `a n`) writes a vx86 image, about
1.5 MB, that starts with `reva --image NAME` or in the browser; it is not a
program you can run by itself, and `makeexename` adds no extension.
`makeexe` is not available. An image starts as the saved executable did:
`onstartup` handlers, then the banner and prompt, with no file being
included. A prompt turned off when saving (piped input) stays off.

`restart` runs the `onexit` handlers, as `bye` does, then starts again
from the same boot or image. The rest of the line it is on is not run.
A file that always calls `restart` restarts for ever (Ctrl-C stops it).

## Floats are doubles inside

The original kept floats in the x87's 80-bit registers; the portable core
keeps them as 64-bit doubles. Loading an 80-bit value (`f@`, CoSy's
stored floats) rounds it to the nearest double, and results can differ
from the original in the last digits of extended precision. Because `f.`
cuts digits instead of rounding, that can show: `30 s>f deg>rad fsin f.`
prints `0.49999`. The rounding mode set with `round.even`, `round.trunc`
etc. applies to conversion to integers (`f>s`, `fround`, `fint`), not to
arithmetic.

## C functions: only a virtual libc

`lib`/`func:` work as before, but the only library is a virtual libc with
`getenv`, `putenv`, `localtime`, `opendir`/`readdir`/`closedir` and
`errno`. Any other library or function
fails to load, and `reva.f` reports it when the function is first called:
`ERROR: could not load <name>`. Native libraries are roadmap item 4.

## Pending (not yet ported)

- *Using a `func` as an xt* (from the old page): applies once foreign
  function calls (`func:`) are ported; the advice stands — wrap the `func`
  in a colon definition before passing its xt to `onexit` or `catch`.
