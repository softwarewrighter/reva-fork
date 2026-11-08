| ls, pp and hl-type are built in: the first use loads the library
| (os/ls, highlight/core).  Run in an empty folder (scripts/in-tempdir.sh).
.needs cr
" b.txt" creat close  " a.f" creat close  " .hidden" creat close
" one.f" creat value h  " : sq dup * ;" h write  h close
ls
.needs cr
" zdir" mkdir  " Adir" mkdir  " zdir/inner.f" creat close
ls
ls zdir
" zdir" (ls)
ls nosuch
ls /reva/lib/highlight
| pp lists a file (plain: the output is not a terminal), and a library
pp one.f
cr " plain" hl-type cr
.needs cr
console-type? .
| the help
help ls
help pp
.s
bye
