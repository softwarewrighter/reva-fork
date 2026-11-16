| A fault in code that a program's own catch runs is caught: catch returns
| the standard throw code, the data stack as at catch time (CoSy's ask 24).
: bad 0 @ ;
1 2 ' bad catch . .s 2drop cr
: div 0 / ;
7 0 ' div catch . .s 2drop cr
| an inner catch gets it, not the outer
: inner ['] bad catch ." inner " . ;
' inner catch ." outer " . cr
| catch catches a throw as before
: boom 5 throw ;
' boom catch . cr
| outside any catch of its own, a fault goes back to the prompt as before
bad
.s cr
bye
