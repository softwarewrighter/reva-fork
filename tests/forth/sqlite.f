| lib/db/sqlite on the CLI: a database file, bound parameters, prepared
| statements, sql_fetch with a callback, errors.  (reva-sqlite; see
| docs/design.md 3.11)
needs db/sqlite
with~ ~db
" work/sqlite-golden.db" 2dup delete sql_open value db
." open " sql_result @ . cr
db " create table pets(id integer primary key, name text, legs integer)" sql_exec
db " insert into pets(name, legs) values(?, ?)" sql_prepare value ins
: add ( a n legs -- ) >r ins 1 2swap sql_bind_string ins 2 r> sql_bind_int ins 0 sql_pexec ;
" cat" 4 add  " bird" 2 add  " fish" 0 add
." last id " db sql_lastid . cr
ins sql_finalize
db " select name, legs from pets where legs >= ? order by name" sql_prepare value sel
sel 1 2 sql_bind_int
: row ( stmt -- f ) dup 0 sql_pcol_text type ." : " 1 sql_pcol_int . cr true ;
sel ' row sql_pexec
sel sql_finalize
: each ( ncols -- f ) ." cols " . 0 sql_getcol$ type ."  has "
	db " select count(*) from pets where legs = 4" sql_fetch# . ." four-legged" cr false ;
." rows " db " select name from pets order by id" ' each sql_fetch . cr
." sum " db " select sum(legs) from pets" sql_fetch# . cr
db " select nope from pets" sql_prepare . ." rc " sql_result @ . db sql_errmsg type cr
db sql_close
" work/sqlite-golden.db" delete
.s cr
