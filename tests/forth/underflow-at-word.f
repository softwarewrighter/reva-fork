| Underflow inside a definition (CoSy's ask 22). f1 takes two cells from
| one and f2 pushes one back, so between words the depth looks fine.
: f1 drop drop ;
: f2 f1 1 ;
| warn (the default) does not see it
1 f2 .s
| diagnose lists the checks; underflow-at-word stops in f1, with f2
diagnose
diagnose underflow-at-word
1 f2
diagnose off
1 f2 .s
| with warn, a line that leaves the stack below empty: the next line
| starts from an empty stack
drop
.s
1 2 .s
bye
