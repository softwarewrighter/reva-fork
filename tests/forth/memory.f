| Heap: allocate free resize
100 allocate dup 0 <> . ioerr @ .
dup 100 'q fill dup 3 type
dup 200 resize dup 0 <> . 3 type
free ioerr @ . cr
