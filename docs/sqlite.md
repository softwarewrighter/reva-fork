# SQLite: `needs db/sqlite`

`lib/db/sqlite` gives Forth programs SQLite databases through the `~db`
words of `lib/db/common`: `sql_open`, `sql_exec`, `sql_prepare`, the
`sql_bind_...` words, `sql_pexec` with `sql_pcol_text` / `sql_pcol_int` /
`sql_pcol_blob`, `sql_fetch` with `sql_getcol$`, `sql_fetch#`,
`sql_fetch$`, `sql_errmsg`, `sql_lastid`, `sql_close`. Programs written for
Reva's SQLite binding run unchanged.

| Host | SQLite | Database files |
|---|---|---|
| CLI (`reva`) | built in (the `reva-sqlite` crate) | real files, paths as for the file words |
| desktop app | built in (the `reva-sqlite` crate, in the app's native side) | the app's folder (`~/Reva`, `--root`, or the package) |
| browser | SQLite's own WebAssembly build, loaded from jsDelivr the first time a program uses SQLite (not redistributed) | the browser's private file system (OPFS) |

In the browser each database runs in memory and is written back to OPFS
(directory `reva-sqlite/`) after every committed change, so it survives a
reload; nothing keeps the file open, so a second tab or a page in the
back/forward cache does not block another. Each change rewrites the whole
file, which suits REPL-sized databases. Without OPFS, databases last for
the session; offline, the first `sql_...` call stops with "SQLite could
not be loaded (offline?)".

In the desktop app a database name is a path in the app's folder, as for
the file words (`notes.db` and `/notes.db` are both `~/Reva/notes.db`); a
name that would leave the folder does not open. The app loads the
folder's files into the VM at start, so a database file (up to 16 MB) is
also visible to the file words: do not write to it with them.

## Example

```forth
needs db/sqlite
with~ ~db
" :memory:" sql_open value db
db " create table pets(name text, legs integer)" sql_exec
db " insert into pets values(?, ?)" sql_prepare value ins
: add ( a n legs -- ) >r ins 1 2swap sql_bind_string ins 2 r> sql_bind_int ins 0 sql_pexec ;
" cat" 4 add  " bird" 2 add
db " select name, legs from pets order by name" sql_prepare value sel
: row ( stmt -- f ) dup 0 sql_pcol_text type ." : " 1 sql_pcol_int . cr true ;
sel ' row sql_pexec
db " select sum(legs) from pets" sql_fetch# .
```
```output
bird: 2
cat: 4
6
```

`:memory:` is a private in-memory database; any other name is a file
(created if missing). `sql_result` holds the last result code (0 OK, 1
ERROR, 100 ROW, 101 DONE) and `sql_errmsg` the connection's last error
message.

## How it works

The library does not call SQLite's C API. It uses the virtual library
`reva-sqlite`, whose functions take handles as small integers and strings
as `( a n )`; the VM turns each call into a *host call* that the host
answers with rusqlite, SQLite through a safe Rust API: the CLI at once,
the desktop app through its page (the VM runs in a worker there, which
pauses until the native side answers). In the browser the VM's worker
answers with SQLite's WebAssembly build (`components/web/crates/reva-web/www/sqlite.js`). See [portability.md](portability.md)
and docs/design.md 3.11.

A statement's first step runs it and keeps its rows, so a SELECT's whole
result is held in memory until the statement is reset or finalized. A
callback given to `sql_fetch` runs on a stack of its own, as SQLite's C
callback did: only the flag it leaves matters.
