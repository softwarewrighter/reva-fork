| Defining words: variable constant create does> alias THROW: ;inline
variable v 42 v ! v @ . cr
7 constant seven seven . : s2 seven seven + ; s2 . cr
create buf 1 , 2 , 3 , buf @ . buf cell+ @ . buf 2cell+ @ . cr
: doubler create , does> @ dup + ; 21 doubler d42 d42 . cr
: sq dup * ; ' sq alias square 9 square . cr
: inc1 1+ ;inline 41 inc1 . cr
| an inline word is copied into its caller, not called
: two inc1 inc1 ; 40 two . cr
: tg THROW_GENERIC ; : tn THROW_NEEDS ;
' tg catch . ' tn catch . cr
last @ >name count type cr
