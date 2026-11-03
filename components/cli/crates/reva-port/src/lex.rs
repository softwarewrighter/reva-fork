//! A Forth source line as pieces: code, the text of string literals, and
//! comments, so the rules look at code and strings only.  Bytes, not
//! UTF-8: Reva sources are often Latin-1.

/// What a piece of a line is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A word (one token).
    Code,
    /// The text of a string literal (`" ..."`, `s" ..."`, CoSy's `` ` x``).
    Str,
    /// `| ...` to the end of the line, or `( ... )`.
    Comment,
}

/// A piece: its kind and its byte range in the line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: Kind,
    pub start: usize,
    pub end: usize,
}

/// The words whose string runs from the next character to the next `"`.
const QUOTE_WORDS: &[&[u8]] = &[b"\"", b"s\"", b"z\"", b".\"", b"c\"", b",\"", b"abort\""];

/// The next token at or after `i`: its range, or `None` at the end.
fn token(line: &[u8], mut i: usize) -> Option<(usize, usize)> {
    while i < line.len() && line[i].is_ascii_whitespace() {
        i += 1;
    }
    let start = i;
    while i < line.len() && !line[i].is_ascii_whitespace() {
        i += 1;
    }
    (start < i).then_some((start, i))
}

/// The pieces of `line`, in order.
pub fn pieces(line: &[u8]) -> Vec<Piece> {
    let mut out = Vec::new();
    let mut i = 0;
    let piece = |kind, start, end| Piece { kind, start, end };
    while let Some((s, e)) = token(line, i) {
        let t = &line[s..e];
        let (next, extra) = match t {
            b"|" => (line.len(), Some(piece(Kind::Comment, s, line.len()))),
            b"(" => {
                let close = find(line, e, b')').map_or(line.len(), |c| c + 1);
                (close, Some(piece(Kind::Comment, s, close)))
            }
            b"`" => match token(line, e) {
                Some((ws, we)) => (we, Some(piece(Kind::Str, ws, we))),
                None => (e, None),
            },
            _ if QUOTE_WORDS.contains(&t) => {
                let from = (e + 1).min(line.len());
                let to = find(line, from, b'"').unwrap_or(line.len());
                out.push(piece(Kind::Code, s, e));
                (to + 1, Some(piece(Kind::Str, from, to)))
            }
            _ => (e, Some(piece(Kind::Code, s, e))),
        };
        out.extend(extra);
        i = next;
    }
    out
}

/// The first `b` in `line` at or after `from`.
fn find(line: &[u8], from: usize, b: u8) -> Option<usize> {
    line.get(from..)?
        .iter()
        .position(|&c| c == b)
        .map(|p| from + p)
}
