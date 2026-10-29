| The virtual libiconv (reva-vm iconv.rs, charset.rs) through
| lib/string/iconv, unchanged: utf>uni / uni>utf, and iconv's own
| results: -1 with errno for an unknown encoding (EINVAL 22), a full
| output buffer (E2BIG 7) and bytes that are not UTF-8 (EILSEQ 84).
needs string/iconv
with~ ~strings
: .bytes ( a n -- ) 0do dup i + c@ . loop drop cr ;
" Grüße €" utf>uni .bytes
" Grüße €" utf>uni uni>utf type cr
z" ebcdic" z" utf-8" iconv_open . errno . cr
z" iso-8859-1" z" utf-8" iconv_open value >latin1
variable in  variable inleft  variable out  variable outleft
create buf 16 allot
: convert ( a n room -- result )
	outleft ! inleft ! in !  buf out !
	>latin1 in inleft out outleft iconv ;
" Grüße" 16 convert . ." -> " buf out @ buf - .bytes
" Grüße" 3 convert . errno . ." in left " inleft @ . cr
" €" 16 convert . errno . cr
create bad 2 allot  $ff bad c!  $41 bad 1+ c!
bad 2 16 convert . errno . cr
>latin1 iconv_close . 99 iconv_close . errno . cr
