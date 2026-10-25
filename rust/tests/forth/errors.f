| Faults are reported with the word they happened in; the REPL recovers.
nosuchword
1 .
: bad 0 @ ; bad
2 .
| a failed definition is rolled back (bad2 must not exist afterwards)
: bad2 1 2 undefinedthing 3 ;
' bad2 .
| stack underflow is detected between words
drop drop drop
3 . cr
| unsupported instruction (x87 fld1) reports the bytes
: fp inline{ d9 e8 } ; fp
4 . cr
