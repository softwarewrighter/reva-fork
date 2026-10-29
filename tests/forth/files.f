| File words on the real file system (run from a scratch directory).
| ioerr is 0 on success and -errno on failure, as on Linux x86.
" files-test.tmp" creat value h  ioerr @ .
" line one" h write  10 pad c! pad 1 h write  ioerr @ .  h close
" files-test.tmp" open/r value r  r fsize .  pad 100 r read .  r close  cr
" files-test.tmp" stat $f000 and $8000 = .  " files-test.tmp" mtime 0 > .  cr
" files-test.tmp" open/rw value w  5 w seek  " ONE" w write  w tell .  w close
" files-test.tmp" slurp type
0 " files-test.tmp" " files-test.ren" rename ioerr @ .
" files-test.tmp" open/r . ioerr @ .  cr
needs os/dir
" files-test.dir" mkdir ioerr @ .  " files-test.dir" chdir ioerr @ .
getcwd + 1- c@ 'r = .
" .." chdir  " files-test.dir" rmdir ioerr @ .  cr
" files-test.ren" delete ioerr @ .  " files-test.ren" delete ioerr @ .  cr
needs crypt/sha1  " /reva/lib/crypt/sha1" sha1file nip .  cr
" /reva/lib/util/nope" open/r . ioerr @ .  cr
bye
