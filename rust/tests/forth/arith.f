| Arithmetic and logic: every word here is machine-code bytes from reva.f
2 3 + . 10 3 - . 6 7 * . -7 2 / . 7 -2 / . cr
5 1+ . 5 1- . 0 1- . cr
$f0 $3c and . $f0 $0f or . $ff $0f xor . 0 invert . cr
0 not . 1 not . -1 not . cr
1 4 << . 256 4 >> . -1 28 >> . 1 33 << . cr
2147483647 1+ . -2147483647 1- 1- . cr
variable v 10 v ! 5 v +! v @ . v ++ v @ . cr
