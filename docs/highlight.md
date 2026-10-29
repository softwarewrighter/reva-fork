# Colour source listings: `pp` and `hl-type`

`needs highlight/core` adds two words that list Forth source in colour:

| Word | Stack | Does |
|---|---|---|
| `pp FILE` | ( -- ) | lists `FILE` in colour; a name that is not a file is tried under `lib/`, so `pp turtle/fractals` lists the library |
| `hl-type` | ( a n -- ) | like `type`, in colour |
| `hl-table` | ( a n -- ) | adds a token table file (later tables win); `lib/highlight/forth.tbl` is loaded already |
| `hl-color?` | ( -- f ) | true when this output shows colour |

`slurp type` and `type` are unchanged: they always print plain text.

```forth
needs highlight/core
" : sq ( n -- n*n ) dup * ; | square" hl-type cr
```

```output
: sq ( n -- n*n ) dup * ; | square
```

That sample prints plain text because the output is not a terminal (see
below); at a terminal or in the browser, `:` and `sq` are bold yellow, the
stack comment and the `|` comment grey, and `dup *` blue.

## Where the colour shows

| Front end | Colour |
|---|---|
| CLI (`reva`) | ANSI escapes when stdout is a terminal, `NO_COLOR` is unset and `TERM` is not `dumb`; on Windows only in Windows Terminal or where `TERM` is set (mintty). `--color always` / `--color never` overrides that. Piped or redirected, `pp` prints exactly what `slurp type` does |
| browser, desktop app, web zip, `reva-serve` | always: the page turns the escapes into styled spans (`www/ansi.js`), themed for light and dark |

Colours are the 16 basic ANSI colours, not fixed RGB values, so a light or
dark terminal (or the page's light or dark theme) picks readable shades.

## The token table, shared with the blog script

What gets which colour comes from **token tables**: plain text files that
both the REPL words and `scripts/forth-highlight.py` (HTML or PNG
figures for the blog and slides) read, so a listing in the REPL and a figure
agree token for token. `lib/highlight/forth.tbl` is the Forth table:

```text
style CLASS SGR CSS     how CLASS looks: ANSI SGR codes (REPLs), then CSS (figures)
words CLASS WORD ...    these words belong to CLASS
| ...                   a comment
```

The tokenizer finds the lexical classes itself, one line at a time:
`com` (`|` to the end of the line), `stk` (`( ... )`), `str` (`"`, `s"` and
`."` strings, `'c` characters), `num` (`123`, `-4`, `$ff`), `colon`
(`:` `;` `;then`) and `def` (the name after `:`). Every other word is
looked up in the tables as written, then in lower case; a word in no table
stays plain. Nothing is evaluated, so listing a file never runs it.

A package adds its own table after Forth's, for its own vocabulary (CoSy
does, with its own `cosy.tbl`):

```text
" mypkg.tbl" hl-table
```

The figure script takes the same tables:

```bash
scripts/forth-highlight.py lib/turtle/fractals --lines 22-30 --png koch.png
scripts/forth-highlight.py FILE --table lib/highlight/forth.tbl --table mypkg.tbl -o out.html
```

With no `--table`, the script reads reva-fork's `forth.tbl` and then every
`*.tbl` beside the script, so a package can ship a copy of the script next
to its own table (ba-cosy-zip-fork does). From a package checkout it finds
reva-fork beside the checkout (`../reva-fork`) or at `$REVA_FORK`.

## Design

- **One tokenizer per surface, one table.** The REPL tokenizer is Rust
  (`components/vlib/crates/reva-vlib/src/highlight.rs`, reached as the virtual library
  `highlight` through `(call)`, like `turtle`); the figure script has a
  Python twin of the same rules. The tables, not the code, say which word is
  which, so adding words never touches either. Unit tests pin the rules
  (`cargo test -p reva-vm highlight`), and the goldens `highlight` and
  `highlight-plain` pin the CLI output.
- **The host decides colour** (`Host::color`), not Forth code: only the
  host knows whether its output is a terminal or a page.
- **Escapes, not a new output channel.** Colour travels as ANSI SGR escapes
  in the ordinary output stream, so the CLI needs nothing more and the page
  needs one small renderer (`ansi.js`, no dependencies, tested by
  `node components/web/crates/reva-web/test/ansi.mjs`). Other programs that print SGR
  escapes now show colour in the page as well.
- **Saved images keep working.** The table list lives in Forth memory, so a
  saved image lists in colour after it restarts.
