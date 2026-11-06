| A fault names the words that called the one at fault, innermost first,
| up to the word the interpreter ran (CoSy's ask 22; docs/gotchas.md).
: a 0 @ ;
: b a 1 ;
: c b 2 ;
c
| the last call of a definition is a jump: d leaves no frame
: d c ;
d
: e d 3 ;
e
| deeper than the 8 callers shown: f6 is cut off
: f1 e 1 ;  : f2 f1 1 ;  : f3 f2 1 ;  : f4 f3 1 ;  : f5 f4 1 ;  : f6 f5 1 ;
f6
| a fault in a word typed at the prompt names no callers
0 @
| inside catch: the chain stops at catch's own frame
' c catch .
.s
bye
