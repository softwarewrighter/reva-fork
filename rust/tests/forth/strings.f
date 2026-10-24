| Strings and memory: parse type count place +place cmp cmpi zt zcount fnvhash
: s 34 parse ;
s hello world" type cr
create b 64 allot
s abc" b place b count type space b c@ . cr
s def" b +place b count type space b c@ . cr
s abc" s abd" cmp . s abd" s abc" cmp . s abc" s abc" cmp . s ab" s abc" cmp . cr
s ABC" s abc" cmpi . s ABC" s abc" cmp . cr
b 1+ zcount . drop cr
s hello" fnvhash . cr
b 64 'x fill b 3 type cr
s xyz" b swap move b 4 type cr
