| argc and argv as in Reva: argv 0 is reva, argv 1 this file, argv 2...
| the words after `--`; an index past the end gives the last (`clamp`).
: .args  argc 0do i . i argv type cr loop ;
argc . cr
.args
argc argv type cr
bye
