| see with the debugger: Reva's x86 disassembler (lib/util/disasm) on
| vx86.  It needs cbw (66 98); core routines in Rust say so.
needs debugger
: sq ( n -- n*n ) dup * ;
: hh ( -- a ) 16 allocate dup 16 0 fill ;
42 constant answer
variable counter
see sq
see hh
see answer
see counter
see allocate
| base is decimal again
10 .
