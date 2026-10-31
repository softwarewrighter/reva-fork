//! Splitting a line into tokens, as `scripts/forth-highlight.py` does.

use crate::table::Table;

/// Space as `forth-highlight.py` sees it (Python's `\s` on one line).
pub fn is_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\r' | 0x0b | 0x0c)
}

/// A decimal or `$`-hexadecimal number.
pub fn is_number(w: &[u8]) -> bool {
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
    while i < n {
        let (class, end) = if is_space(line[i]) {
            let j = line[i..]
                .iter()
                .position(|&b| !is_space(b))
                .map_or(n, |j| i + j);
            ("", j)
        } else {
            word_token(line, i, &mut naming, table)
        };
        out.push((class, i, end));
        i = end;
    }
    out
}

/// The token starting with the word at `i`: its class and end.  A `:`
/// makes the next word a definition's name (`naming`).
pub fn word_token<'t>(
    line: &[u8],
    i: usize,
    naming: &mut bool,
    table: &'t Table,
) -> (&'t str, usize) {
    let n = line.len();
    // Up to the closing byte `c` (or the end of the line).
    let upto = |from: usize, c: u8| {
        line[from..]
            .iter()
            .position(|&b| b == c)
            .map_or(n, |j| from + j + 1)
    };
    let j = line[i..]
        .iter()
        .position(|&b| is_space(b))
        .map_or(n, |j| i + j);
    let w = &line[i..j];
    let (class, end): (&str, usize) = match w {
        b"|" => ("com", n),
        b"(" => ("stk", upto(i + 1, b')')),
        b"\"" | b"s\"" | b".\"" => ("str", upto(j, b'"')),
        _ if *naming => {
            *naming = false;
            ("def", j)
        }
        b":" => {
            *naming = true;
            ("colon", j)
        }
        b";" | b";then" => ("colon", j),
        [b'\'', _] => ("str", j),
        _ if is_number(w) => ("num", j),
        _ => (table.class_of(w).unwrap_or(""), j),
    };
    (table.lexical(class), end)
}
