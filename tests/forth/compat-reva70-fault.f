| compat/reva70: a fault in a callback add-callback queued is reported and
| gui-main-loop goes on, as if it returned gui-default (CoSy's ask 24: Bob's
| Tui.f buttons do their work this way).
needs compat/reva70
needs ui/gui
with~ ~ui
variable runs
dialog[
  " Bad" button[ { { 1 runs +! 0 @ gui-default } add-callback gui-default } action ]w
  " Done" button[ { { ." queued closes" cr gui-close } add-callback gui-default } action ]w
]d " reva70" title
show gui-main-loop
." loop ended after " runs @ . ." faults, depth " depth . cr
bye
