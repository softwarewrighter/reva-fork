| A fault in an IUP callback is reported and the dialog goes on, the
| callback taken as returning IUP_DEFAULT (CoSy's ask 24): Bad faults, Ok
| still counts, the close box ends the loop; the stack is as without faults.
needs ui/gui
with~ ~ui
variable n
: bad callback  1 n +!  -1 @ drop  gui-default ;
: ok callback  10 n +!  gui-default ;
: d  dialog[ hboxs[ " Bad" button[ action: bad ]w " Ok" button[ action: ok ]w ]c ]d " F" title show gui-main-loop ;
d n @ . depth . cr
