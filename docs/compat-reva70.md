# Reva 7.0 compatibility: `needs compat/reva70`

reva-fork runs Reva 2011.2 (`src/reva.f`). Some programs were written for
the older Reva 7.0, CoSy among them. Where 7.0 and 2011.2 differ in what
such a program uses, `needs compat/reva70` gives it 7.0's behaviour. The
layer is opt-in, and the program loads it first. Everything else stays
2011.2, so the Reva demos are unchanged.

```text
needs compat/reva70      | first, before the program's own files
needs ui/gui             | now Reva 7.0's ui/gui (and ui/iup)
```

## What it changes

| Area | 2011.2 | With the layer (7.0) |
|---|---|---|
| `needs ui/gui`, `needs ui/iup` | 2011.2's libraries | 7.0's, from `lib/compat/r70/ui/`: `func:` IUP bindings, and `add-callback`'s queue, which `gui-main-loop` empties between events |
| `value` | a macro | an ordinary word, so `: >value value ;` works |
| `>defer`, `make`, `undo`, `save-default` | missing | 7.0's deferred-word words (`undo` goes back to what `save-default` recorded, else `noop`) |

A library loaded through the layer is recorded as loaded, so 2011.2 code
that needs it later (reva-fork's `ui/notebook`) finds it. The notebook
runs the same on 7.0's `ui/gui`, which gained 2011.2's attribute words
(`attr`, `attr?`, `global-attr?`, `]dw`).

## Changes to 7.0's libraries for the port

Reva is public domain, so 7.0's libraries are here, with each change
marked `| reva-fork:`:

- **Saving.** 7.0's ui/gui made `save-gui` patch a Windows executable.
  Here `save` writes an image, so `save-gui` is `save`. (Its Linux branch,
  which reva-fork would take, did not compile in 7.0 either.)
- **The event loop.** 7.0's `gui-main-loop` polled `IupLoopStep` every
  millisecond. Here `IupLoopStep` waits for the next event (no busy loop),
  and the loop stops before stepping once a queued callback returned
  `gui-close`.
- **The library names** 7.0 loads IUP by (`libiup.so`, `iup.dll`, ...)
  are served by the IUP virtual library, as 2011.2's are.

## What the layer does not do

It does not emulate Windows. A program that hard-codes `C:\` paths, splits
paths at `\`, or runs `del`/`copy` through the shell is migrated to
portable Forth instead (`pathsep`, relative paths, reva-fork's file words);
that is the job of a separate migration tool (saga step
`local-port-migrate`).

## CoSy

With the layer in place of ba-cosy-zip-fork's `compat708.f`, CoSy loads,
and Bob's Job.f and Tui.f, unchanged, open their window. F6 then evaluates
a line into the result pane (checked from a scratch build; this
repository holds no CoSy code). Two Windows-only lines remain for the
migration: Tui.f's window title splits a path at `\`, and `save` backs up
with `del` and `copy`.
