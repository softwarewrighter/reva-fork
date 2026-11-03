# Porting Reva programs from Windows: `reva --port-check`

Many Reva programs were written on Windows. reva-fork runs the same Forth
on every host, but it does not pretend to be Windows: there is no `C:`
drive and no `cmd.exe`. Paths are always VM paths, with `/` as the
separator. Portable systems work the same way. Java's `File.separator`,
Python's `os.sep` and Node's `path.join` build paths from parts, and all
of them accept `/` on Windows too. A hard-coded `C:\` path is a bug in the
program, not something the runtime translates.

`reva --port-check` finds what ties a program to Windows and writes a
portable copy for review:

```text
reva --port-check FILE...              # report, as FILE:LINE:COL
reva --port-check --fix DIR FILE...    # also write migrated copies to DIR
```

With `--fix`, each FILE is copied to DIR with the safe fixes applied, never
over the original (DIR must not hold the sources). The report also goes to
`DIR/port-report.txt`. The copies keep the original line ends, so
`diff FILE DIR/FILE` shows only the fixes. The exit status is 0 when nothing
is left to do, 1 when findings remain, and 2 for a usage or file error.

## What it finds

Comments are never changed. The rules look at code and string literals:

| Rule | Example | Fix |
|---|---|---|
| `drive-path` | `" C:\apps\notes\notes.txt"`, `` ` C:/4thCoSy/CoSy/MinJob.csy`` | `./notes.txt`: the file beside the program, for a person to confirm |
| `path-separator` | `" \" split`, `s" \" ss`, `` ` \ toksplt``: a string that is just `\`, given to a word that splits or joins paths | `/`. Windows reported paths with `\`; reva-fork reports them with `/` on every host |
| `windows-command` | `" del notes.bak" shell`, `copy`, `dir`, `type`, `start`, `cd`, `md` ... run through `shell` | by hand, with the portable word: `delete`, `rename`, `slurp` and `write`, `os/dir`, the host's opener |
| `bat-file` | `" setup.bat"` | by hand: do its steps in Forth |

Backslashes that are not paths are left alone: word names (`at\`, `cut\`),
banners (`." \/ ops \/ "`) and deliberate converters (`` ` \ ` / ,L ssr``).

## Example

From `tests/forth/windows-only.f` (golden `port-check`):

```text
windows-only.f:4:16: drive-path: the drive exists only on Windows: a path relative to the program, ./notes.txt
    fix: C:\apps\notes\notes.txt -> ./notes.txt
windows-only.f:5:37: path-separator: paths in reva-fork use / on every host (Windows reported them with \)
    fix: \ -> /
windows-only.f:7:16: windows-command: `del` is a Windows shell command; instead: delete ( a n -- )
    by hand: : tidy       " del notes.bak" shell ;
port-check: 1 file(s): 2 fixed in the copies, 4 to do by hand
```

## CoSy

Run on Bob's sources (from a scratch copy; this repository holds no CoSy
code), the checker finds the separator splits in CoSy.f and in Tui.f's
window title, the `C:/4thCoSy` path in `newJob`, and `save`'s `del`/`copy`
backup, plus `dir`, `cd`, `start` and a `.bat` in Furniture.f. With the safe
fixes and [the Reva 7.0 layer](compat-reva70.md), Bob's flow loads Tui.f
and opens its window, and F6 evaluates into the result pane. The `del`/`copy`
backup is the CoSy lane's to rewrite in Forth, with Bob's agreement.
