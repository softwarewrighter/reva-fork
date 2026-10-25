| catch/throw use ESP-based frames exactly as newcore.asm did
: boom 7 throw ;
: safe ['] boom catch ;
safe . cr
: fine 1 2 + ;
' fine catch . . cr
: nested ['] safe catch . ;
nested cr
| a throw with 0 is a no-op
: zero 0 throw 5 ; ' zero catch . . cr
| the data stack is restored to its depth at catch time
1 2 3 ' boom catch . . . . cr
