| notebook.f -- the TUI notebook (lib/ui/notebook; docs/ui-model.md)
|
| Type a Forth line in the top pane, press Ctrl-J or F6 (fn+F6 on a Mac):
| what the line prints, any error and the stack it leaves show in the
| lower pane.  Done or the x in the top border closes it; its text is kept
| in notebook.txt.  The same two words work at any prompt, the live
| demo's included.

needs ui/notebook
notebook
