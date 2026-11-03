| add-callback (lib/ui/gui): the queued xt runs, and its gui-close ends the
| main loop, as in CoSy's Tui.f ({ { ... gui-default } add-callback gui-default }).
needs ui/gui
with~ ~ui
variable runs
dialog[
  " Again" button[ { { 1 runs +! ." queued ran " runs @ . cr gui-default } add-callback gui-default } action ]w
  " Done" button[ { { ." queued closes" cr gui-close } add-callback gui-default } action ]w
]d " add-callback" title
show gui-main-loop
." loop ended after " runs @ . cr
bye
