| A program written for Reva on Windows (reva-fork's own sample for
| reva --port-check; golden port-check).
| The data lives in C:\apps\notes  (comments are never changed)
: data-file  " C:\apps\notes\notes.txt" ;
: notes-dir  ( a n -- a n )  2dup " \" split ;   | splits a path Windows reported
: backup     " copy notes.txt notes.bak" shell ;
: tidy       " del notes.bak" shell ;
: launch     " start setup.bat" shell ;
: banner ." \/ notes \/ " cr ;                   | backslashes that are not paths
