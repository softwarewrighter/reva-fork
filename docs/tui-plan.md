# Plan: portable forms and TUI layer (IUP-compatible subset)

Status: plan, 2026-10-26. Capabilities and testable features only.
Implementation design starts after the source refactor; nothing here
assumes a crate, module or file layout. Forth-level names (`needs
ui/gui`, `IupDialog`, `K_F6`) are the contract with existing programs,
so they are used freely.

## Why, and why here

Reva ships a generic GUI vocabulary (`needs ui/gui`, built on `needs
ui/iup`, which calls the IUP C library through the x86 FFI). CoSy's real
interface, `Tui.f` and `Job.f`, is written against it. Today neither
library loads: the port has no native C libraries, IUP cannot run in
WASM, and the project rule is "native code only through built-in
bindings" (see [parity.md](parity.md), GUI decision).

The layer belongs in reva-fork, not in ba-cosy-zip-fork: the
vocabulary is Reva's, it is generic, and other programs can use it.
ba-cosy-zip-fork keeps only CoSy's layout (`Tui.f`, `Job.f`) and must be
able to load them unchanged (its ask 1, `docs/reva-fork-asks.md`).

Approach: a built-in implementation of the subset of the IUP API and of
`ui/gui` that real programs use, drawn by three interchangeable
backends: HTML forms (browser and desktop app, through the same host UI
service as turtle graphics), a full-screen terminal UI (CLI), and a
headless backend for tests. This revises the parity.md note that IUP
programs are not counted toward parity, for this subset only.

Rejected for now: a new native widget word set with `Tui.f` / `Job.f`
ported to it. It would fork Bob's code and leave other `ui/gui` programs
broken. It can still be added later on top of the same widget model.

## Goals

1. `Tui.f` and `Job.f` load and run unmodified on every host: browser,
   desktop app, CLI terminal, headless.
2. One widget model, three backends; a program cannot tell which one it
   runs on except through documented, optional attributes.
3. The REPL is not lost: leaving the GUI loop returns to it, and errors
   in callbacks are reported, not fatal.

Non-goals: full IUP; canvases and CD/IM drawing (turtle covers 2D);
OpenGL; native look-and-feel; pixel-exact layout.

## Non-functional requirements

- **Portability.** Same behaviour in WASM (browser Web Worker), the
  desktop app, and the native CLI on Linux, macOS and Windows. No C
  library, no platform GUI toolkit, no dlopen.
- **Licensing.** Permissive only (see Licensing below); no GPL-family
  dependency for the terminal or web backends.
- **Accessibility and usability.** Web: real form controls, labels tied
  to fields, ARIA roles, full keyboard operation, visible focus, works
  at 200% zoom and in dark mode. Terminal: usable over SSH, no meaning
  carried by colour alone, honours `NO_COLOR`, redraws on resize, works
  in 80x24. Every F-key binding has a documented alternative chord for
  hosts that swallow F-keys.
- **Testability.** Every feature below is checked on the headless
  backend by scripted events and golden output, in the project's normal
  gate; web and terminal backends get smoke tests driving the same
  scripts.
- **Upgradability.** Messages between the VM and a UI backend carry a
  protocol version; unknown attributes are ignored with one warning per
  name, so newer programs degrade instead of failing. Saved images that
  hold widget-using code stay loadable across releases.
- **Performance.** Callback round trip under 50 ms locally; setting or
  reading a 1 MB text value under 100 ms; label updates after every
  evaluation do not flicker.
- **Serviceability and traceability.** A debug switch logs every widget
  call, attribute and event; errors name the widget, attribute and
  callback involved.

## Scope: what CoSy uses

Inventoried read-only from ba-cosy-zip-fork `sample/CoSy/Tui.f` and
`sample/CoSy/Job.f` (dev ffb5e15).

| Kind | Used |
|---|---|
| `ui/gui` construction | `dialog[ ... ]d`, `hbox[ ... ]c`, `label[`, `editbox[`, `button[`, `]w`, `spacer`, `file-open-dialog[ ... ]fd` |
| `ui/gui` attributes | `size`, `expand`, `tip`, `title`, `attr: FONT`, `attr: MULTIPLEFILES`, `insert` (INSERT), `topmost` (TOPMOST) |
| `ui/gui` values and text | `setval`, `getval`, `getcaret`, `setcaret`, `caret-y?`, `line-text?` |
| `ui/gui` lifecycle | `show-xy`, `popup`, `hide`, `destroy`, `gui-main-loop`, `gui-close`, `gui-default` |
| `ui/gui` callbacks | `action`, `set-callback` with `K_F1`..`K_F12`, `K_sF6`, `K_sF11`, `K_cF6`, `K_cF9`, `K_cS`; `callback`, `add-callback` (defer work to the Forth environment) |
| Raw IUP (Job.f) | `IupDialog`, `IupMultiLine`, `IupSetAttribute`, `IupStoreAttribute`, `IupGetAttribute`, `IupShowXY`, `IupDestroy` |
| Raw attributes | `VALUE`, `TITLE`, `SIZE`, `FONT`, `CARET`, `TABSIZE`, `EXPAND`, `X`, `Y` |
| Outside the layer | `shell>` (Ctrl-F6) and `www` (F8) run host commands; they stay under the existing `--allow-shell` policy |

Prerequisite outside the layer: `needs ui/gui` first fails on `iconv`,
so a built-in iconv (UTF-8 and Latin-1) comes first.

Widget set derived from this: dialog (top-level window), vbox (implied
by `dialog[`), hbox, label, multi-line text, button, filler, file-open
dialog. Anything else in `ui/gui` is out of scope until a program needs
it; using it gives a clear "not supported in this port" error.

## Backends

- **Web (browser and desktop).** Each dialog is a panel in the page, a
  split pane, or a separate window (same choices as turtle graphics).
  Widgets are HTML form controls; key callbacks are bound on the focused
  control. The file dialog lists the host's virtual files (localStorage
  in the browser, real files on desktop).
- **Terminal (CLI).** Full-screen, character cells. Dialogs are stacked
  or tabbed screens; boxes lay out as rows and columns; multi-line text
  is an editor pane with the existing line-editing keys; buttons are
  focusable and activated with Enter or a mnemonic. Falls back to an
  error naming the headless backend when stdout is not a terminal.
- **Headless.** No display. Holds the widget tree in memory, accepts
  scripted events (key, click, text entry, file-dialog answer) and
  dumps the tree and values as text for goldens.

## Features and acceptance criteria

Each feature is a testable statement. "On all backends" means the
headless golden plus a web and a terminal smoke test.

- **F1 Load.** `needs ui/gui` and `needs ui/iup` load on every host.
  *Accept:* `reva -e 'needs ui/gui' -e bye` exits 0 on CLI, web and
  desktop; no native-library note is printed.
- **F2 Tree building.** `dialog[`, `hbox[`, `label[`, `editbox[`,
  `button[`, `spacer`, `]w`, `]c`, `]d` build the tree the program
  describes. *Accept:* headless dump of `Tui.f`'s `define-dialog1`
  matches a reviewed golden (labels, three edit boxes, nine buttons, in
  order).
- **F3 Values.** `setval`/`getval` and `VALUE` set and read text,
  round-tripping any byte string the encoding allows. *Accept:* set and
  get of ASCII, Latin-1 and a 1 MB value compare equal on all backends.
- **F4 Attributes.** `TITLE`, `SIZE`, `FONT`, `EXPAND`, `TIP`,
  `TABSIZE`, `X`, `Y`, `TOPMOST` are stored and readable; backends apply
  what they can (SIZE in IUP character units maps to columns and rows;
  FONT maps to a family and size, ignored in the terminal). *Accept:*
  get after set returns the set string on all backends; a documented
  table says which attributes each backend renders.
- **F5 Caret and lines.** `CARET` ("line,col"), `getcaret`,
  `setcaret`, `caret-y?`, `line-text?` and `INSERT` behave as IUP's.
  *Accept:* scripted caret moves then `line-text?` return the expected
  line; `INSERT` at caret yields the expected value.
- **F6 Actions.** A button's `action` runs its xt when activated by
  click, Enter, or the test script. *Accept:* clicking "Reset stack" in
  `Tui.f` empties the stack and updates the stack label.
- **F7 Key callbacks.** `set-callback` with every key name CoSy uses
  fires on the focused widget only. *Accept:* scripted F6 on the text
  pane evaluates the line under the caret and shows the result in the
  res pane; the same key in the res pane uses the res pane's binding.
- **F8 Return codes.** `gui-default`, `gui-close`, `gui-continue` and
  IUP's ignore code have IUP's meaning. *Accept:* a callback returning
  `gui-close` ends `gui-main-loop`; the REPL prompt returns.
- **F9 Deferred callbacks.** `add-callback` work runs in the normal
  Forth environment after the event returns, in order. *Accept:* nested
  `{ { ... } add-callback gui-default }` from `Tui.f` runs exactly once,
  after the outer callback.
- **F10 Main loop.** `gui-main-loop` processes events until
  `gui-close`; in the browser it never blocks the page, and the stop
  button interrupts it. *Accept:* web smoke test: open `Tui.f`, press
  stop, REPL responds.
- **F11 Several dialogs.** More than one dialog can be shown at once
  and destroyed independently (`Tui.f`'s main dialog plus `Job.f`'s
  `.r` window). *Accept:* `rShow`, edit, `rGet`, `rCloseWdo` round
  trips on all backends.
- **F12 Show, hide, position.** `show-xy`, `popup`, `hide`, `destroy`;
  `X`/`Y` are honoured where windows exist and readable everywhere.
  *Accept:* show at (40,60), read `X` and `Y` back.
- **F13 File dialog.** `file-open-dialog[ ... ]fd popup getval` returns
  the chosen path(s), `MULTIPLEFILES` separated as IUP does, or empty
  on cancel. *Accept:* headless answer script and web smoke test.
- **F14 Callback errors.** An exception in a callback is reported in
  the REPL output with the callback's key or widget, and the loop keeps
  running. *Accept:* a callback that divides by zero leaves the dialog
  usable.
- **F15 Key alternatives.** Each F-key and Ctrl chord has a documented
  alternative where the host reserves it (browser F1, F5, F11, F12,
  Ctrl-S; terminals without F-keys). *Accept:* each alternative fires
  the same callback in a scripted test; table in the user docs.
- **F16 CoSy end to end.** ba-cosy-zip-fork can drop its `Tui.f` /
  `Job.f` skip and run `go`. *Accept:* its CoSy demo opens the Tui,
  F6-evaluates `2 3 +`, shows `5` in res, saves with Ctrl-S (or its
  alternative), and restores the panes after restart.

## Phasing

1. **M0 Prerequisites.** Built-in iconv; F1 with headless only.
2. **M1 Widget model + headless.** F2-F9, F11, F13, F14 against
   headless goldens; raw `Iup*` calls (Job.f) and `ui/gui` words share
   the model.
3. **M2 Web backend** (browser and desktop together, via the UI
   service). F10, F12, F15 web half; Job.f's `.r` window first, then
   all of `Tui.f`.
4. **M3 Terminal backend.** All features on the CLI; F15 terminal half.
5. **M4 Hardening.** Accessibility review, backend attribute table,
   user docs, parity.md update; F16 with ba-cosy-zip-fork.

Each milestone is releasable on its own; later milestones add backends,
not semantics.

## Licensing

IUP is from Tecgraf/PUC-Rio and is under the MIT licence; this
repository already carries it as `LICENSE.IUP` (copyright 1994-2010),
and [licenses.md](licenses.md) lists IUP as MIT. The official copyright
page (tecgraf.puc-rio.br/iup, "Copyright") could not be fetched from
the cloud session that wrote this plan; check it before M1.

The plan reimplements a compatible API subset: same function names,
attribute names and behaviour, with no IUP source or binaries bundled
or linked. Reimplementing an API for compatibility does not copy IUP's
code; the MIT licence would permit copying anyway, with its notice
kept. Rules:

- No IUP source, headers or binaries in the new layer.
- If any IUP code or documentation text is copied (for example
  attribute descriptions in help text), keep its MIT notice and record
  it in licenses.md.
- `ui/gui` and `ui/iup` themselves are Reva wrappers (Danny Reinhold)
  under Reva's terms and are already in the tree.
- Terminal and web dependencies, if any, must pass the existing licence
  check (no GPL family).

This is an engineering summary, not legal advice.

## Open questions

1. While `gui-main-loop` runs, should the REPL stay usable (Tui as a
   second front end) or be suspended as on Windows Reva?
2. Web layout default for dialogs: in-page panel, split pane, or a
   separate window?
3. Which alternative chords for F-keys the browser reserves, and should
   they be configurable per package?
4. Terminal: tabs versus stacked screens for several dialogs; mouse
   support?
5. Should `www` (F8) open a browser tab in the web host, and is that
   covered by the `--allow-shell` policy or a separate one?
6. Beyond CoSy, which other `ui/gui` programs (if any) should be in
   scope, to fix the widget set before M1?
7. Fonts: CoSy asks for `APLSans::10`; ship a permissively licensed APL
   font in the web host, or fall back to monospace?
