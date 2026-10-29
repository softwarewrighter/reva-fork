//! # Syntax highlighting: the virtual library `highlight`
//!
//! `pp` and `hl-type` (`lib/highlight/core`) list Forth source in colour.
//! The words to colour come from token tables, plain text files that the
//! blog's `scripts/forth-highlight.py` reads too, so the REPLs and the
//! blog figures agree (`lib/highlight/forth.tbl`; a package adds its own,
//! such as CoSy's).  A table line is one of
//!
//! ```text
//! style CLASS SGR CSS     how CLASS looks: ANSI SGR codes, then CSS
//! words CLASS WORD ...    these words belong to CLASS
//! | ...                   a comment
//! ```
//!
//! Later lines win, so a later table can recolour a word.  Lexical classes
//! need no words: `com` (`|` to the end of the line), `stk` (`( ... )`),
//! `str` (`"`, `s"`, `."` strings and `'c` characters), `num`, `colon`
//! (`:` `;` `;then`) and `def` (the name after `:`).  A word is looked up
//! as written, then in lower case.
//!
//! Output is the source with each coloured token wrapped in `ESC[SGRm` and
//! `ESC[0m`; the bytes between tokens are copied unchanged.  When the host
//! says colour is off ([`Host::color`]: the CLI writing to a pipe, say)
//! the source is written exactly as `type` would.  The browser page turns
//! the escapes into spans (`www/ansi.js`).
//!
//! Forth reaches it like `turtle`: `" highlight" lib`, then
//! `hl_type ( src n tables count -- 0 )`, where `tables` holds `count`
//! pairs of cells (address, length) of table texts, and
//! `hl_color ( -- flag )`.

use std::collections::HashMap;

use crate::fault::Fault;
use crate::gates::Gate;
use crate::host::Host;

/// Handle `(lib)` returns for the virtual highlight library ("HILT").
pub const HL_HANDLE: u32 = 0x4849_4c54;

/// Library names accepted for it.
pub const HL_NAMES: &[&str] = &[
    "highlight",
    "libhighlight",
    "libhighlight.so",
    "highlight.dll",
];

/// At most this many tables in one call.
const MAX_TABLES: u32 = 16;

/// The gateway of highlight function `name`.
pub fn lookup(name: &str) -> Option<Gate> {
    match name {
        "hl_type" => Some(Gate::HlType),
        "hl_color" => Some(Gate::HlColor),
        _ => None,
    }
}

/// Token tables merged: SGR codes per class, and each word's class.
#[derive(Default, Debug)]
pub struct Table {
    sgr: HashMap<String, String>,
    words: HashMap<Vec<u8>, String>,
}

impl Table {
    /// Add the lines of one table text (later lines win).
    pub fn add(&mut self, text: &str) {
        for line in text.lines() {
            let mut f = line.split_ascii_whitespace();
            match (f.next(), f.next()) {
                (Some("style"), Some(class)) => {
                    let sgr = f.next().unwrap_or("0");
                    self.sgr.insert(class.to_string(), sgr.to_string());
                }
                (Some("words"), Some(class)) => {
                    for w in f {
                        self.words.insert(w.as_bytes().to_vec(), class.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    fn class_of(&self, w: &[u8]) -> Option<&str> {
        self.words
            .get(w)
            .or_else(|| self.words.get(&w.to_ascii_lowercase()))
            .map(String::as_str)
    }
}

/// The classes the tokenizer finds by itself.
const LEXICAL: &[&str] = &["com", "stk", "str", "num", "colon", "def"];

/// Space as `forth-highlight.py` sees it (Python's `\s` on one line).
fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | 0x0b | 0x0c)
}

fn is_number(w: &[u8]) -> bool {
    let dec = w.strip_prefix(b"-").unwrap_or(w);
    let hex = w.strip_prefix(b"$");
    (!dec.is_empty() && dec.iter().all(u8::is_ascii_digit))
        || hex.is_some_and(|h| !h.is_empty() && h.iter().all(u8::is_ascii_hexdigit))
}

/// The tokens of one line (no newline): (class, start, end), where class
/// `""` is plain (spaces, or a word in no class).  Mirrors `tokens()` in
/// `scripts/forth-highlight.py`.
pub fn tokens<'t>(line: &[u8], table: &'t Table) -> Vec<(&'t str, usize, usize)> {
    let (mut out, mut i, n, mut naming) = (Vec::new(), 0, line.len(), false);
    // Tokens up to the closing byte `c` (or the end of the line).
    let upto = |from: usize, c: u8| {
        line[from..]
            .iter()
            .position(|&b| b == c)
            .map_or(n, |j| from + j + 1)
    };
    while i < n {
        if is_space(line[i]) {
            let j = line[i..]
                .iter()
                .position(|&b| !is_space(b))
                .map_or(n, |j| i + j);
            out.push(("", i, j));
            i = j;
            continue;
        }
        let j = line[i..]
            .iter()
            .position(|&b| is_space(b))
            .map_or(n, |j| i + j);
        let w = &line[i..j];
        let (class, end): (&str, usize) = match w {
            b"|" => ("com", n),
            b"(" => ("stk", upto(i + 1, b')')),
            b"\"" | b"s\"" | b".\"" => ("str", upto(j, b'"')),
            _ if naming => {
                naming = false;
                ("def", j)
            }
            b":" => {
                naming = true;
                ("colon", j)
            }
            b";" | b";then" => ("colon", j),
            [b'\'', _] => ("str", j),
            _ if is_number(w) => ("num", j),
            _ => (table.class_of(w).unwrap_or(""), j),
        };
        // Lexical classes are named by the tokenizer, so give them the
        // table's own &str (or plain, if the table has no style for them).
        let class = if LEXICAL.contains(&class) {
            table
                .sgr
                .get_key_value(class)
                .map_or("", |(s, _)| s.as_str())
        } else {
            class
        };
        out.push((class, i, end));
        i = end;
    }
    out
}

/// `src` highlighted with ANSI SGR escapes, line by line.
pub fn render(src: &[u8], table: &Table) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() * 2);
    for (k, line) in src.split(|&b| b == b'\n').enumerate() {
        if k > 0 {
            out.push(b'\n');
        }
        for (class, a, b) in tokens(line, table) {
            match table.sgr.get(class) {
                Some(sgr) if !class.is_empty() => {
                    out.extend_from_slice(format!("\x1b[{sgr}m").as_bytes());
                    out.extend_from_slice(&line[a..b]);
                    out.extend_from_slice(b"\x1b[0m");
                }
                _ => out.extend_from_slice(&line[a..b]),
            }
        }
    }
    out
}

reva_macros::machine_trait! {
    /// The virtual library `highlight`.
    pub trait Highlight {
        /// A call into the virtual highlight library, or `None` for another gate.
        fn highlight_call(
            &mut self,
            host: &mut dyn Host,
            g: Gate,
            args: &[u32],
        ) -> Result<Option<u32>, Fault> {
            let a = |i: usize| args.get(i).copied().unwrap_or(0);
            match g {
                Gate::HlColor => Ok(Some(u32::from(host.color()).wrapping_neg())),
                Gate::HlType => {
                    let src = self.mem.slice(a(0), a(1))?.to_vec();
                    if !host.color() {
                        host.write(&src);
                        return Ok(Some(0));
                    }
                    let mut table = Table::default();
                    for t in 0..a(3).min(MAX_TABLES) {
                        let p = a(2).wrapping_add(t * 8);
                        let (ta, tn) = (self.mem.read_u32(p)?, self.mem.read_u32(p + 4)?);
                        table.add(&String::from_utf8_lossy(self.mem.slice(ta, tn)?));
                    }
                    host.write(&render(&src, &table));
                    Ok(Some(0))
                }
                _ => Ok(None),
            }
    }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORTH: &str = include_str!("../../../../../lib/highlight/forth.tbl");

    fn classes(line: &str) -> Vec<(String, String)> {
        let mut t = Table::default();
        t.add(FORTH);
        tokens(line.as_bytes(), &t)
            .into_iter()
            .filter(|(c, a, b)| !c.is_empty() || !line[*a..*b].trim().is_empty())
            .map(|(c, a, b)| (c.to_string(), line[a..b].to_string()))
            .collect()
    }

    fn pairs(v: &[(&str, &str)]) -> Vec<(String, String)> {
        v.iter()
            .map(|(c, s)| (c.to_string(), s.to_string()))
            .collect()
    }

    #[test]
    fn definition_line() {
        assert_eq!(
            classes(": sq ( n -- n*n ) dup * ; | square"),
            pairs(&[
                ("colon", ":"),
                ("def", "sq"),
                ("stk", "( n -- n*n )"),
                ("stack", "dup"),
                ("stack", "*"),
                ("colon", ";"),
                ("com", "| square"),
            ])
        );
    }

    #[test]
    fn strings_numbers_control() {
        assert_eq!(
            classes(r#"10 0do ." hi there" 'x emit $ff IF loop"#),
            pairs(&[
                ("num", "10"),
                ("ctl", "0do"),
                ("str", r#"." hi there""#),
                ("str", "'x"),
                ("", "emit"),
                ("num", "$ff"),
                ("ctl", "IF"),
                ("ctl", "loop"),
            ])
        );
    }

    #[test]
    fn unterminated_runs_to_end_of_line() {
        assert_eq!(classes("( open"), pairs(&[("stk", "( open")]));
        assert_eq!(classes("\" open"), pairs(&[("str", "\" open")]));
    }

    #[test]
    fn later_table_wins_and_render_wraps_tokens() {
        let mut t = Table::default();
        t.add(FORTH);
        t.add("style cosy 1;32 color:#98c379\nwords cosy dup rho\n");
        assert_eq!(tokens(b"dup", &t)[0].0, "cosy");
        let out = render(b"7 rho\nfoo", &t);
        assert_eq!(
            String::from_utf8(out).unwrap(),
            format!(
                "\x1b[{n}m7\x1b[0m \x1b[1;32mrho\x1b[0m\nfoo",
                n = t.sgr["num"]
            )
        );
    }

    #[test]
    fn crlf_and_plain_bytes_are_kept() {
        let t = Table::default();
        let src = b"a  b\r\n\tc \xff\n";
        assert_eq!(render(src, &t), src.to_vec());
    }
}
