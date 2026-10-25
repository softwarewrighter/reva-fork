# Reva documentation

Documentation for the portable Reva core, rebuilt from the implementation in
[`rust/`](../rust/). It replaces the old wiki in [`www/`](../www/), which
describes the deprecated x86 distribution (see [parity.md](parity.md) for
what has been carried over and what has not, yet).

| Page | Covers |
|---|---|
| [Getting started](getting-started.md) | building, the `reva` command, the browser build |
| [Gotchas](gotchas.md) | pitfalls, old and new |
| [Parity](parity.md) | old wiki topics and example programs vs. the portable core |
| [Example parity](parity-examples.md) | generated status of every program in `examples/` |
| [Desktop app](desktop.md) | `reva-desktop`: build, run, files on disk, packages |
| [Portability](portability.md) | what stops on the portable core (native libraries, `shell`, x87/SIMD) and why |
| [Licenses](licenses.md) | third-party files, their licenses, and the no-copyleft check |
| [Design](../rust/DESIGN.md) | how the portable core works and why |

## Conventions: every sample is executed

A Forth sample is a `forth` block followed by an `output` block:

```forth
: sq dup * ;
7 sq .
```
```output
49
```

`rust/scripts/doc-check.sh` feeds each `forth` block to a fresh `reva` and
compares the result with the `output` block (ignoring trailing spaces); the
regression suite runs it over these pages. A page cannot drift from the
implementation without a test failing. When adding a sample, run it first and
paste what `reva` actually prints, then check that it is *right*.
