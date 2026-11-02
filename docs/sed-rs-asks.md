# What reva-fork needs from sed-rs

Requests from using [sed-rs](https://github.com/sw-cli-tools/sed-rs) to
edit this repository's files (2026-11-01), most valuable first. Each has
the case that hit it and what would have fixed it.

Today an in-place edit is written as the user's undoable pattern:

```bash
mv file file.orig && sed-rs -e 's/old/new/' < file.orig > file
# wrong? mv file.orig file      right? rm file.orig
```

## 1. In-place editing with a backup: `-i[SUFFIX]`

```bash
sed-rs -i.orig -e 's/old/new/' file      # file.orig keeps the original
sed-rs -i -e 's/old/new/' a.rs b.rs      # several files, no backup
```

- Write the result to a temporary file in the same directory, then rename
  it over the original, so a reader never sees a half-written file.
- If the script does not parse, or any error occurs, leave every file
  untouched and exit non-zero before writing anything.
- Keep the file's permissions.

**Why:** a script that sed-rs rejected ("address ranges are not
supported") still left `> CHANGES.md` truncating the file, and the
command went on to commit an empty CHANGES.md. With `-i`, a failed
script changes nothing.

## 2. Address ranges, and "first match only"

```bash
sed-rs -e '0,/^## 2026-11-01$/s//## 2026-11-01\n\n- new line/'   # first match only
sed-rs -e '10,20s/a/b/'          # lines 10 to 20
sed-rs -e '/^def: /,/^=$/d'      # from one pattern to the next
sed-rs -e '5,/^$/s/x/y/'         # from a line to a pattern
```

**Why:** CHANGES.md lines go under the first header for the day; there
is no way to address only the first match today.

## 3. Literal (fixed-string) patterns: `-F`, or a flag on `s`

```bash
sed-rs -F -e 's/  ." F6 (fn+F6)" cr/  ." Cmd-Enter or F6" cr/'
sed-rs -e 's/a (b) + c/x/L'      # L: the pattern is a literal string
```

Neither the pattern nor the replacement is a regex: `( ) [ ] . * + ? \ |`
and `&` mean themselves.

**Why:** most edits here replace exact code (Forth, Rust, Markdown),
full of these characters. Writing `[(]`, `[+]`, `[.]` is error-prone;
`\(` and `\+` silently became operators through the BRE conversion, and
two of four edits matched nothing.  Later (2026-11-01): `||` in a Rust
pattern matched as an empty alternation and spliced the replacement into
the wrong place; `\{` became an interval (`repetition quantifier expects
a valid decimal`); and `^| ` (a Markdown table row) read `|` as
alternation, so the empty `^` branch matched every line and the
replacement was prepended to all 544 lines of a file.  Without `-i`, a failed run under `> file` also leaves
the file empty, so each of these needed a restore from the `.orig` copy.

## 4. Report edits that changed nothing: `--strict`

```bash
sed-rs --strict -i -e 's/old/new/' file   # exit 1, naming the script, if it matched nothing
```

A warning on stderr without `--strict` would also help.

**Why:** an edit that silently matches nothing looks like success; it was
caught only by a `diff` afterwards.

## 5. Show the change: `--diff`

```bash
sed-rs -i --diff -e 's/old/new/' file     # edit, and print a unified diff
sed-rs --diff -e 's/old/new/' < file      # dry run: only the diff
```

**Why:** every edit was followed by a separate `diff` to confirm it.

## What already works well

- `\n` in a replacement, to insert a line after a match.
- Several `-e` scripts in one run.
- Rust regex classes and repetition (`[0-9]{4}`, `\s`, `+`).
- The same behaviour on macOS and Linux, which is why the scripts here use
  sed-rs and never the system `sed`.
