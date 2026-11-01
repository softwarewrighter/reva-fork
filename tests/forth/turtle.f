| turtle graphics: geometry, state and SVG of small fractals, from the CLI
needs turtle/fractals
: sq 4 0do 100 forward 90 right loop ;
clear sq xy@ . . heading@ . segments . cr
| tpush/tpop save the whole turtle; loops only work inside definitions
: fan 3 0do tpush 50 forward tpop 30 right loop ;
clear penup 10 20 setxy pendown 45 right fan
xy@ . . heading@ . segments . cr
| the bulk path: absolute headings, one step each
create hs 0 , 90 , 180 , 270 ,
clear 25 step! hs 4 path xy@ . . segments . cr
2 koch-demo segments . cr  2 sierpinski-demo segments . cr  6 dragon-demo segments . cr
3 hilbert-demo segments . cr  4 tree-demo segments . cr  3 plant-demo segments . cr
| depth caps
99 hilbert-demo segments . cr
2 hilbert-demo " /tmp/reva-turtle-test.svg" svg-save . cr
" /tmp/reva-turtle-test.svg" slurp type
bye
