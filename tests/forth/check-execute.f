| execute of what is not code (CoSy's ask 22). Without the check, 0
| execute jumps to 0 and the fault names only that address.
: go 0 execute ;
: run go 1 ;
run
| with it, the execute stops, naming the word that executed and its callers
diagnose execute
run
: q2 12345 execute ;
q2
| words, quotations and vectors still run
: ok1 ['] dup execute ;
5 ok1 .s
: quot { 1 + } execute ;
5 quot . cr
defer dv  ' 1+ is dv
5 dv . cr
diagnose off
bye
