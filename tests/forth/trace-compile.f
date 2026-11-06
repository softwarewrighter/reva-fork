| diagnose compile: what is compiled into a definition, and who compiled
| it when it was not the interpreter (CoSy's ask 22, item 4).
diagnose compile
: sq dup * ;
| prior compiles the old word by its class, which compiles while the
| compiler is on: run from a macro during t's definition, allocate
| compiles core allocate into t and gives back its argument (1001)
: allocate prior allocate 1 + ;
macro : m 1000 allocate . ; forth
: t m ;
diagnose off
| the old word's xt, executed, runs whatever the state (an address)
' allocate constant (allocate)
: allocate2 (allocate) execute 1 + ;
macro : m2 1000 allocate2 0 <> . ; forth
: t2 m2 ;
bye
