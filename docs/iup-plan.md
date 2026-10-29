# Plan: IUP programs (and CoSy's TUI) on reva-fork

High-level plan for running Reva's IUP GUI programs on the portable core,
CoSy's "Textual User Interface" (`Tui.f`) first among them. What needs to
be done, in what order, and how we will know it works. The low-level
design (components, crates, modules, files) comes later, after the
sw-checklist work, within the rules in CLAUDE.md. Saga steps:
`local-ui-model`, `local-iup`, `local-ui-yew`, `local-tui-cosy`.

## Goal

Existing Reva GUI programs run **unchanged** on reva-fork: in the browser,
in the desktop app and on the live demo. The Forth side
(`lib/ui/iup`, `lib/ui/gui`, the programs) is not edited.

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
3. **The renderer** (`local-ui-yew`). Rust/Yew components draw the model
   in the page: dialogs as windows within the page or in the split view,
   controls as themed HTML, user input (clicks, edits, keys including
   function keys, caret and selection) sent back as events. The desktop
   app gets it through the same page.
4. **CoSy's TUI end to end** (`local-tui-cosy`). `Tui.f` running
   unchanged in the desktop app and the browser; whatever it still needs
   fixed here; the CoSy lane told what to drop.
5. **The rest of the examples**, in order of how many programs a feature
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

- **The CLI.** It has no window. Options: no GUI on the CLI (a clear
  stop, as now); or a terminal renderer of the same model (a real
  text-mode TUI), later. To decide when the model exists.
- **Look.** Match IUP's native look, or the REPL page's theme (light and
  dark)? The plan assumes the page's theme.
- **Windows.** One dialog at a time inside the page, several as in-page
  windows, or separate native windows in the desktop app (as the turtle
  graphics window)?
- **Drawing (CD) and images.** Map onto the turtle canvas's drawing
  surface, or a separate canvas element?
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
