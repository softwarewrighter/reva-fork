| Stack words (EAX = TOS, ESI = data stack pointer)
1 2 swap . . cr
1 2 over . . . cr
1 2 3 rot . . . cr
1 2 3 -rot . . . cr
1 2 nip . cr
1 2 tuck . . . cr
1 2 3 third . . . . cr
4 5 2dup . . . . cr
9 dup . . cr
1 2 3 _nip . . cr
1 2 drop . cr
1 2 2drop 3 . cr
: rt 1 >r 2 r@ . r> . . ; rt cr
