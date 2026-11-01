| notebook.f -- a small notebook in the TUI (reva-fork; docs/ui-model.md)
| In the desktop app: copy it to the app's folder, then  include notebook.f
|
| Type a Forth line in the top pane, then press F6 (fn+F6 on a Mac
| keyboard): the line under the cursor is evaluated, and what it prints,
| any error, and the stack it leaves show in the lower pane.  Forth
| needs a space between words: 6 7 * not 6 7*.
| Enter starts a new line, Tab moves between the panes and the Done
| button, Done or the x in the top border closes the notebook.
| `demo` opens it again.

needs ui/gui
with~ ~ui

variable text   variable res   variable status

| The line under the cursor of the top pane.
: line ( -- a n )  text @ caret-y? line-text? rot drop ;

| While a line is evaluated, what it prints goes to the lower pane.
variable old-emit   variable old-type   variable emit-pad
| `attr` makes its strings C strings in place (a 0 after them), so what
| is typed (word names in the dictionary, for `words`) is copied first.
create pane-buf 256 allot
: pane-chunk ( a n -- )  >r pane-buf r@ move  res @ pane-buf r> " INSERT" attr drop ;
: pane-type ( a n -- )
  repeat
    2dup 255 min dup >r pane-chunk
    swap r@ + swap r> -
  dup 0 > while 2drop ;
: pane-emit ( c -- )  emit-pad c!  emit-pad 1 pane-type ;
: to-pane ( -- )
  ['] emit @ old-emit !  ['] type @ old-type !
  ['] pane-emit is emit  ['] pane-type is type ;
: to-repl ( -- )  old-emit @ is emit  old-type @ is type ;

| What the line left on the stack, oldest first (what is above depth d0).
variable d0
: .new ( -- )
  depth d0 @ - dup 0 > if dup 0 do dup i - pick . loop then drop ;

| F6: evaluate the line; its output, any error and the stack below.
| A plain word, not a `callback` one: the TUI runs it on the data stack
| (as IUP called C callbacks) and puts the stack back afterwards, so
| words like `help` and `.s` work as at the REPL.
: f6
  res @ " " setval drop
  depth d0 !
  to-pane
  line ['] eval catch ?dup if ." error " . cr then
  cr ." stack: " .new
  to-repl
  res @ " 1,1" " CARET" attr drop
  status @ " evaluated" title drop
  gui-default ;

: notebook ( -- dialog )
  dialog[
    " Type a line, then F6 (fn+F6)" label[ expand-horizontal dup status ! ]w
    editbox[ expand
      ['] f6 z" K_F6" set-callback
      dup text ! ]w
    editbox[ " x64" size expand-horizontal dup res ! ]w
    hbox[ " Done" button[ { gui-close } action ]w ]w
  ]d " FULLxFULL" size " Reva notebook" title ;

| The top pane's text is kept in notebook.txt (in the app's folder):
| loaded when the notebook opens, saved when it closes.
: notes ( -- a n )  " notebook.txt" ;
: load-notes ( -- )  notes slurp dup if text @ -rot setval drop else 2drop then ;
: save-notes ( -- )  text @ getval rot drop  notes creat >r r@ write r> close ;

: demo ( -- )  notebook load-notes show gui-main-loop save-notes destroy ;

demo
