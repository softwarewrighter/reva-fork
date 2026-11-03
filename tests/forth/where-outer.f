| errors say where (golden error-where): this file includes where-inner.f
: bad ( -- ) 0 @ ;
include tests/forth/where-inner.f
." back in where-outer.f" cr
drop
." after the underflow" cr
1 2 7 8 bad
