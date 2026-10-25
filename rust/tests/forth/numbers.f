| Number parsing and printing: bases, prefixes, chars, doubles, (.r)
12345 . -12345 . 0 . cr
$ff . $FF . %1010 . &17 . #99 . cr
hex ff . 7fffffff . -1 . decimal cr
binary 1011 . decimal 2147483647 . -2147483647 . cr
| -2147483648 does not parse: single uses a signed-overflow check, as in newcore
-2147483648
'A . 'z . cr
42 6 (.r) type '| emit cr
2147483648
1234L . . cr
-1L . . cr
