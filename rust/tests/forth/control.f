| Control flow compiled to x86 jumps: if/else/then, 0if, =if, <if, >if, ;then
: sgn dup 0if drop 0 ;then 0 <if -1 ;then 1 ;
-5 sgn . 0 sgn . 7 sgn . cr
: t if 1 else 2 then ; 0 t . 5 t . cr
: eq 3 =if 33 ;then 44 ; 3 eq . 4 eq . cr
: ne 3 <>if 33 ;then 44 ; 3 ne . 4 ne . cr
: gt 3 >if 33 ;then 44 ; 2 gt . 4 gt . cr
| recursion: the trailing call becomes a jmp (tail call), no return stack growth
: cd dup . 1- dup if cd ;then drop ; 5 cd cr
: deep dup 0if ;then 1- deep ; 100000 deep . cr
| 00; exits on zero (keeping it); 0; drops it and exits
: z 00; 42 ; 0 z . 1 z . . cr
: z2 0; 42 ; 0 z2 1 z2 . . cr
