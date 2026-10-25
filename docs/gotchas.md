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
was before the `include`. Check the first `ERROR:` line in the output.

## Times are UTC

`time&date` and `localtime` report UTC: the portable core has no time-zone
database. The clock comes from the host (the browser host has none yet, so
there it reads as the Unix epoch).

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
code writes is kept in `localStorage` (about 5 MB per site, so keep demo
files small). Empty directories are not kept, and modification times there
read as 0.

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
