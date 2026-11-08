needs ui/gui
with~ ~ui

variable clicks   variable shown
: count-it callback
    1 clicks +!  shown @ clicks @ (.) attr: TITLE drop  gui-default ;
: quit callback  gui-close ;

: counter ( -- )
    0 clicks !
    dialog[
      vboxs[
        " 0" label[ dup shown ! ]w
        hboxs[
          " Count" button[ action: count-it ]w
          " Quit" button[ action: quit ]w
        ]c
      ]c
    ]d " Counter" title
    show gui-main-loop ;
counter
