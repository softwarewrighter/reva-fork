| compat/reva70: Reva 7.0's words and ui/gui on the 2011.2 core.
needs compat/reva70
| 7.0's value is an ordinary word, so a word can call it.
: >value value ;
42 >value answer  answer . cr
| >defer, make and undo on a deferred word.
defer greet
' noop >defer greet
make greet ." hello from make" cr ;
greet
undo greet  greet ." (undone)" cr
| 7.0's ui/gui: add-callback queues the xt for gui-main-loop.
needs ui/gui
with~ ~ui
variable runs
dialog[
  " Again" button[ { { 1 runs +! ." queued ran " runs @ . cr gui-default } add-callback gui-default } action ]w
  " Done" button[ { { ." queued closes" cr gui-close } add-callback gui-default } action ]w
]d " reva70" title
show gui-main-loop
." loop ended after " runs @ . cr
' handle-pending-events 0 <> . cr
bye
