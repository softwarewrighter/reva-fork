| `}` compiles its own `ret` before `;then`, so `;;` must not take a byte
| inside the previous call's offset for a `call` (E8): it made it E9, and
| the call landed a byte into its word (examples/data/parray.f faulted).
| Here nops before the quotation are counted so that the offset of
| its call to `bump` has E8 as its low byte: `bump` must run as written
| and leave the stack as it was.
variable cnt
: bump  1 cnt +! ;
| nops (90) so that a call compiled at here + n + 5 (after `{`'s jmp) has
| an offset ending in E8
: pad-e8 ( -- n )  ['] bump here - 10 - $e8 - 255 and ;
: nops ( n -- )  0 ?do $90 1, loop ;
: q  [ pad-e8 nops ]  { bump } ;
q execute q execute
cnt @ . depth . cr
bye
