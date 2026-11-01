| A notebook in the shape of CoSy's Tui.f, scripted by tests/ui/notebook.ui:
| F6 evaluates the line under the caret into the result pane, F11 inserts
| a note, a button asks for a file, the close box ends the main loop.
needs ui/gui
with~ ~ui

variable text  variable res  variable status

: insert ( handle a n -- handle )  " INSERT" attr ;
: line ( -- a n )  text @ caret-y? line-text? rot drop ;

| A callback word, as Reva wrote them for C; its stack is its own.
: f6 callback
  line eval (.) res @ -rot setval drop
  status @ " evaluated" title drop
  gui-default ;

: notebook ( -- dialog )
  dialog[
    " ready" label[ dup status ! ]w
    editbox[ " 200x40" size expand ['] f6 z" K_F6" set-callback
      { text @ " | note" insert drop gui-default } z" K_F11" set-callback
      dup text ! ]w
    editbox[ " 200x16" size dup res ! ]w
    hbox[
      " File..." button[
        { file-open-dialog[ ]fd popup getval ." file: " type cr destroy gui-default } action ]w
    ]w
  ]d " Notebook" title ;

: go  notebook show gui-main-loop >r
  ." text: " text @ getval type drop cr
  ." result: " res @ getval type drop cr
  r> destroy ." depth " depth . cr ;
go bye
