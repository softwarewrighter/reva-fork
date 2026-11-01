# The UI model

The first step of [running IUP programs](iup-plan.md) (saga step
`local-ui-model`): a model of the IUP world in Rust, with no VM and no
rendering. The `iup` virtual library (`local-iup`) answers Reva's IUP
calls over it; the cell-grid renderer (`local-ui-grid`) and the terminal
backend (`local-ui-terminal`) draw it. IUP's API is reimplemented from its
documentation; nothing of IUP is used or redistributed
([licenses](licenses.md)).

## Components

| Component | Crate | What |
|---|---|---|
| `ui` | reva-ui-base | the vocabulary: element kinds by IUP's class names, IUP's key codes and names, the change stream and its text form |
| | reva-ui-text | editable text: characters, caret, selection; lines and columns; IUP's position forms; editing keys |
| | reva-ui-tree | the element tree: creating, appending, destroying; attributes with IUP's inheritance and defaults; callbacks by name; relations |
| | reva-ui | the model a program's UI is: text attributes computed from the edited text, showing and hiding dialogs, the focus, events and the callbacks they run |
| `ui-layout` | reva-ui-layout | IUP's layout in character cells: natural sizes, SIZE, EXPAND, boxes |

None has a dependency outside the repository, so the VM can use them as
it is (docs/design.md 4: no dependencies).

## Scope

What `lib/ui/iup` binds (about 75 C functions), what `lib/ui/gui` makes of
it (136 words), and what the 20 programs in `examples/iup` and CoSy's
`Tui.f` use (70 of the words; Tui.f 33) fixed the scope.

**Elements** (reva-ui-base's `Kind`): dialog, vbox, hbox, zbox, fill,
frame, radio, cbox, sbox, tabs, label, button, toggle, text, multiline,
list, canvas, matrix, menu, item, submenu, separator, file dialog,
message dialog, timer, image, user. Containers hold children (a dialog
one); menus hold menu entries; nothing holds a dialog.

**Attributes** are stored by name, upper case, as strings, the way IUP
keeps them. ACTIVE, BGCOLOR, FGCOLOR, FONT, TABSIZE and VISIBLE are
inherited from the parents; EXPAND, ALIGNMENT, GAP, MARGIN, VALUE (a
toggle's), TOPMOST, DIALOGTYPE, VISIBLECOLUMNS, VISIBLELINES and RUN have
defaults per kind. A text or multiline element keeps its text apart from
the attributes, and reads and applies these from it:

| Attribute | Read | Set |
|---|---|---|
| VALUE | the text | replaces it, caret at the start |
| CARET | `lin,col` (multiline) or `col`, from 1 | moves the caret (columns past a line's end go to its end) |
| CARETPOS | from 0 | moves the caret |
| SELECTION | `lin1,col1:lin2,col2` or `col1:col2`, the end after the last character; none when nothing is selected | selects; `ALL`, `NONE` |
| SELECTIONPOS | `pos1:pos2` from 0 | selects |
| SELECTEDTEXT | the selected text | replaces the selection, if any |
| INSERT | | types the text at the caret |
| APPEND | | adds the text at the end (a multiline puts a newline first) |
| COUNT, LINECOUNT, LINEVALUE | characters, lines, the caret's line | |

`gui`'s `caret?`, `caret-y?`, `selection?`, `selected-text?` and
`line-text?` (SELECTION `n,1:n,1000`, then SELECTEDTEXT) work on these.

**Callbacks** are stored by name with the program's function (for Reva, a
Forth word's address). A key goes to the focused element: the callback
named for the key (`K_F6`, `K_sF6`, `K_cS`), looked up on the element
and then its parents, else `K_ANY` with the key code. With no callback,
or one returning `DEFAULT` (`gui-default`) or `CONTINUE`, the key does
what it does by default: Tab and Shift-Tab move the focus, Enter and space
click a button, other keys edit a text element. A click runs a button's,
toggle's or item's ACTION (a toggle's with its new state); the close box
runs CLOSE_CB, else hides the dialog. A callback returning `CLOSE` ends
the main loop, which also ends when no dialog is shown.

**Keys** have IUP 3's codes: a character or a named key (`K_F1` is
`0xFFBE`, `K_HOME` `0xFF50`, `K_DEL` `0xFFFF`, ...) with bits for Shift,
Ctrl, Alt and Sys; a letter with Ctrl or Alt is upper case (`K_cS`), and
Shift with a character is the character.

**The change stream.** Every change to what is shown is recorded:
created, appended, detached, destroyed, an attribute set or removed, a
dialog shown (with its position, modal or not) or hidden, the focus
moved. A text element's edits record its VALUE, CARET and SELECTION. A
renderer that applies the changes in order has the model's state; in
another process or language it reads them as lines of text (reva-ui-base's
`codec`: `attr 3 VALUE =hello\sworld`).

## Layout in cells

The renderer draws a grid of character cells in every host, so the layout
is computed in cells, with IUP's rules:

- **Natural sizes** as the TUI draws the elements: a label its title, a
  button `[ OK ]`, a toggle `[x] Title`, a text field its VISIBLECOLUMNS
  (10) plus brackets, a multiline its VISIBLECOLUMNS by VISIBLELINES (20
  by 4) in a one-cell frame, a list its widest item by its items, framed;
  a dialog, a frame and tabs a one-cell border with the title in it.
- **SIZE** is in IUP's character units (a quarter of a character across,
  an eighth down; `400x100` is 100 by 13 cells), RASTERSIZE, MARGIN and
  GAP in pixels (8 by 16 to a cell), CMARGIN and CGAP in character units.
  Either replaces the natural size in its direction.
- **Boxes** put their children one after another at their natural sizes,
  with GAP between and MARGIN around, and share the space left equally
  among the children that expand along the box. Across, an expanding
  child takes the whole box, the others go where ALIGNMENT puts them.
- **EXPAND**: YES, HORIZONTAL, VERTICAL or NO (controls' default). A
  container expands where its EXPAND allows (YES by default) and one of
  its children expands; a fill expands along its box.

A dialog is laid out at its SIZE, else its natural size, at most the
window; the rectangles are relative to the dialog's corner.

## The virtual library `iup`

Step `local-iup`: component `iup`, crate reva-vlib-iup, answers the C
functions `lib/ui/iup` binds over the model, which the machine keeps
(`Machine::ui`, host state like open files: not in an image). Programs
load it as before (`" libiupall.so" lib iup`, `func:`), and `lib/ui/gui`
and the programs run unchanged.

- **No pointers cross.** Elements are the model's handles (small
  integers); strings are copied in, and returned strings are copied into
  VM buffers kept per element and attribute.
- **Addresses.** The functions are not gates: `(func)` gives each an
  address in the null page (`0x400` + 4 per entry of the table in
  `names.rs`), where nothing can run, and `(call)` looks it up. The table
  is append-only, as the gates are, since images store the addresses.
- **The event loop.** `IupMainLoop` and `IupPopup` (of a dialog) jump to
  the gate `IupLoop`, which sends the host the changes since the last time
  and asks for an event: the host call `ui` `wait`, bytes = the change
  lines, answered with one event line (`key K_F6`, `key 65`, `click 5`,
  `focus 3`, `close 1`). A host that answers later (the desktop's page)
  pauses the VM there, as SQLite calls do. The loop ends when no dialog is
  shown (a popup's: when it is hidden) or a callback returns `CLOSE`; the
  `(call)` then returns 0.
- **Callbacks** run as IUP ran them: a C call into the Forth word, its
  arguments and the element pushed as C pushes them, returning to the gate
  `IupResume`. So both Reva's `callback` words (their own stack) and plain
  `{ ... }` blocks (the result left in EAX) work. Afterwards the data stack
  (ESI and EAX) is as it was, as C's convention keeps those registers. A
  callback can itself wait for the host (SQLite, a file dialog, a popup).
  A fault inside one leaves the loops, back to the interpreter.
- **File dialogs.** `IupPopup` of a file dialog asks the host (`ui`
  `file`, bytes = DIALOGTYPE, TITLE, FILTER, DIRECTORY, MULTIPLEFILES),
  answered with the path (bytes) or a cancel (an integer); the dialog's
  VALUE and STATUS (`0`, `-1`) follow.

**Hosts.** The CLI has no UI yet (`local-ui-terminal`); with `reva
--ui-script FILE` it answers from a script (crate reva-ui-script) and
prints what is shown (`ui< ` change lines, `ui> ` events):

```text
# a comment
key K_F6          a key, by IUP's name or code
type 2 3 +        the characters, one key each
click 5           a button, toggle or menu item (by handle)
focus 3
close 1           a dialog's close box
file /tmp/a.txt   the answer to a file dialog
cancel            a file dialog cancelled
```

The goldens `iup-hello-world`, `iup-simple-button` and `iup-notebook` (a
Tui.f-shaped notebook, `tests/forth/iup-notebook.f`) run this way. The
browser and the desktop app refuse `ui` calls until the grid renderer
(`local-ui-grid`).

## Not yet

Left to the steps that need them: drawing a menu bar (a dialog's MENU
reserves its row), switching tabs and zbox children, lists' selection
and editing, the matrix and its `IupMat*` functions, timers firing,
`IupLoopStep` waiting for an event (it returns `DEFAULT`), canvases
(`local-ui-canvas`). IUP calls `K_ANY` before the key's own callback;
the model calls one or the other, which is what the programs do.
