# Plan: IUP programs (and CoSy's TUI) on reva-fork

High-level plan for running Reva's IUP GUI programs on the portable core,
CoSy's "Textual User Interface" (`Tui.f`) first among them. What needs to
be done, in what order, and how we will know it works. The low-level
design (components, crates, modules, files) comes later, after the
sw-checklist work, within the rules in CLAUDE.md. Saga steps:
`local-ui-model`, `local-iup`, `local-ui-grid`, `local-ui-terminal`,
`local-tui-cosy`, `local-ui-canvas` (stage 1; the terminal backend before
`Tui.f`, so the first TUI demo runs in all three hosts); `local-ui-panes`
(stage 2).

## Goal

Existing Reva GUI programs run **unchanged** on reva-fork: in the browser,
in the desktop app and on the live demo, and the Notebook (below) in the
CLI's terminal too. The Forth side (`lib/ui/iup`, `lib/ui/gui`, the
programs) is not edited.

## Two stages (the user, 2026-10-30)

1. **The Notebook: parity first.** What Reva and CoSy did with IUP.
   CoSy's `Tui.f` is a notebook: one dialog with three edit panes (the
   text you type, the result of evaluating it, a state pane), a stack
   status line, a row of buttons, and F-keys (F6 evaluates the current
   line, shift-F6 the selection, F8/F9 send a line elsewhere, others
   insert dates or save the text into CoSy's dictionary). Reva's own
   `examples/iup` are single dialogs of widgets. Stage 1 runs both in the
   browser, the desktop app, and (without graphics) the CLI's terminal.
2. **Panes: a longer-term goal** that builds on stage 1, not the other
   way round. A tmux/Emacs-inspired layout: split horizontally and
   vertically, zoom one pane to the full window and back; an editor pane
   (load, save, evaluate a line into a REPL), a tail pane (follow a file
   or another pane's output), several REPLs, IUP dialogs in panes; the
   same layout in the browser, the desktop app and the terminal.

## Decisions (with the user, 2026-10-28)

- **Reimplement, do not wrap or redistribute.** IUP (Tecgraf/PUC-Rio) is
  a C toolkit; the core cannot call native code, and we do not ship it.
  We implement, in Rust, the part of IUP's C API that Reva's libraries
  call, as a *virtual library* (the pattern of iconv and SQLite).
- **Rendered with Rust/Yew**, in the web page and the desktop app (which
  shows the same page).
- **In reva-fork.** `lib/ui/gui` is a Reva library; every IUP program
  benefits, CoSy is one user. ba-cosy-zip-fork then only drops its "no
  GUI" patches.
- **After the component restructuring** (saga steps `local-components`
  to `local-comp-front-ends`), so the new code starts in the new shape.

## What Reva programs use (the scope)

| Layer | What | Size |
|---|---|---|
| `lib/ui/iup` | binds the IUP C libraries (`libiup`, `libiupcd`, `libiupcontrols`, `libiupgl`) through `lib`/`func:` | ~75 C functions |
| `lib/ui/gui` | the Forth vocabulary (Danny Reinhold): `dialog[ … ]d`, `vbox[`/`hbox[`, `button[`, `label[`, `editbox[`, `attr:`, `action:`, `gui-main-loop` … | 136 words |
| programs | 20 in `examples/iup/`, and CoSy's `Tui.f` | 21 programs |

The 21 programs use **70 of the 136 words**. Every one builds a dialog
and runs `gui-main-loop`; most use buttons, labels, boxes and actions:

- **Core** (most programs): `dialog[ ]d`, `button[`, `label[`, `hbox[`
  `hboxs[` `vbox[` `vboxs[`, `spacer`, `frame[`, `title`, `size`,
  `expand`, `tip`, `attr:`, `action:`, `action[`, `show`, `popup`,
  `gui-main-loop`, `getval`/`setval`.
- **Text editing** (Tui.f and 2 examples): `editbox[`, `text[`, `caret?`,
  `caret-y?`, `selection?`, `selected-text?`, `line-text?`, key callbacks
  (`K_F1`…`K_F12`, `K_cS`, `key-any-cb:`).
- **Dialogs**: `file-open-dialog[`, `file-save-dialog[`, message boxes.
- **Menus and bars**: `menu[`, `menu-item[`, `menu-separator`,
  `]submenu`, `toolbar[`.
- **Other controls**: `toggle[`, `radio[`, lists, tabs, `matrix[`
  (IupControls), timers (`timer`, `set-time`, `start-timer`).
- **Drawing**: `canvas[` with the CD drawing library (4 programs), images
  (`ui/gui-iup-img`, 3), OpenGL (1).

CoSy's `Tui.f` needs 33 words: the core set, text editing with F-keys,
`file-open-dialog[`, `popup`/`show-xy`/`hide`/`destroy`, `topmost`, and
one direct `IupStoreAttribute`.

## Work, in order

1. **UI model** (`local-ui-model`). A Rust model of IUP's world, with no
   VM and no rendering: the element tree (dialogs, boxes, controls),
   attributes by IUP's names and its inheritance rules, callbacks by
   name, the layout IUP computes (natural size, `EXPAND`, `SIZE`,
   margins and gaps), and a stream of changes a renderer applies.
   Tested on its own.
2. **The `iup` virtual library** (`local-iup`). The C functions above,
   answered in the VM over that model; handles are integers, strings
   copied, so no pointer crosses. Events: `IupMainLoop` and `IupPopup`
   wait for the user, as a host call that pauses the VM (the way SQLite
   calls pause in the desktop app); when an event arrives, the Forth
   callback runs, and its result (`IUP_DEFAULT`, `IUP_CLOSE`, ...)
   decides what happens next. A headless host drives events in tests.
3. **The renderer: one TUI everywhere** (`local-ui-grid`; decided with
   the user 2026-10-31). The model lays out into a grid of character
   cells (text, colour, cursor), pure Rust with goldens on the grid. Thin
   backends draw it: a monospace grid element in the web page (the
   desktop app shows the same page), and crossterm in the CLI's terminal
   (step 5). Keys (function keys included), mouse and resize go back as
   events. crossterm alone cannot do it: it needs a real terminal and does
   not build for wasm; the grid makes the look the same in all three.
4. **CoSy's TUI end to end** (`local-tui-cosy`). `Tui.f` running
   unchanged in the desktop app and the browser; whatever it still needs
   fixed here; the CoSy lane told what to drop.
5. **The terminal backend** (`local-ui-terminal`): the grid in the CLI
   through crossterm (raw mode, keys, mouse, resize, redrawing changed
   cells): `Tui.f` and the simple examples in a terminal.
6. **Graphics** (`local-ui-canvas`; browser and desktop only). Reva drew
   through IUP's `canvas[` and Tecgraf's CD library (`lib/ui/cd`: lines,
   boxes, arcs, polygons, text, colours, clipping, images), repainting
   from the canvas's redraw callback. CD becomes a virtual library that
   draws into the turtle's display list (components/canvas), shown by a
   canvas pane; parity: `binary-clock`, `brownian`, `cd-test`. Not CD's
   PDF/printer contexts, and not OpenGL.
7. **The rest of the examples**, in order of how many programs a feature
   unblocks: menus and toolbars, toggles, radios, lists and tabs, timers,
   file and message dialogs, the matrix; then drawing (the CD library on
   the turtle canvas's drawing surface), images; OpenGL last, if at all.

## How we will know it works

- Goldens (CLI, headless host): building and querying dialogs, attribute
  values, callbacks fired by scripted events, for `examples/iup/`'s
  simple programs and a scripted Tui-like session.
- The web smoke test: the renderer's DOM for a known model; events back.
- Manual checks in Chrome and the desktop app on the Mac, with
  screenshots, for each phase.
- The parity report: the 20 IUP examples move from "could not load
  IupOpen" to running, phase by phase.

## Open questions

- **The CLI.** Decided: the same cell grid through crossterm (step 5),
  without graphics.
- **Look.** Decided: a text-mode TUI, the same in every host, coloured
  with the REPL page's themes (light and dark). HTML widgets maybe later.
- **Windows.** One dialog at a time inside the page, several as in-page
  windows, or separate native windows in the desktop app (as the turtle
  graphics window)?
- **Drawing (CD) and images.** Decided: the turtle's display list, in a
  canvas pane (step 6).
- **Licensing of names.** IUP's API names and behaviour are reimplemented
  from its documentation (MIT); nothing of its code. To record in
  docs/licenses.md when the work starts.

## Risks

- IUP's layout rules are detailed; programs that depend on exact pixel
  sizes may look different. Mitigation: implement IUP's documented
  algorithm, compare with screenshots of the original where we have
  them.
- Callback-heavy programs (timers, key-any) stress the pause/resume
  path; the bench guard and the smoke test watch it.
- Scope creep across 136 words and IUP's controls library: phase 5 is
  ordered by programs unblocked, and stops where the examples stop.
