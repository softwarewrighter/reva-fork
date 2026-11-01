| highlight: pp and hl-type (lib/highlight/core), run with --color always
| and --color never (work/reg-rs/highlight*.rgt)
needs highlight/core
." color: " hl-color? . cr
" : sq ( n -- n*n ) dup * ; | square" hl-type cr
" 10 0do .\" hi\" 'x emit $ff loop 3 forward" hl-type cr
pp tests/forth/highlight.f
pp no/such/file
bye
