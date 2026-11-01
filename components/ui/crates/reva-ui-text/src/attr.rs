//! IUP's forms for positions and ranges (CARET, SELECTION).
//!
//! In a multiline element a position is `"lin,col"`, otherwise `"col"`;
//! both count from 1, and a column past the end of its line is the end of
//! the line.  A range is `"pos1:pos2"`, `pos2` the position after the last
//! character in it; `"ALL"` and `"NONE"` are ranges too.

use crate::pos::{index, line_col};
use crate::text::Text;

/// Position `i` of `t` as IUP writes it.
pub fn format_pos(t: &Text, i: usize) -> String {
    let (line, col) = line_col(&t.chars, i);
    if t.multiline {
        format!("{},{}", line + 1, col + 1)
    } else {
        format!("{}", col + 1)
    }
}

/// The position `s` names in `t` ("lin,col" or "col"), clamped to the text.
pub fn parse_pos(t: &Text, s: &str) -> Option<usize> {
    let num = |s: &str| s.trim().parse::<usize>().ok().map(|n| n.max(1) - 1);
    match s.split_once(',') {
        Some((line, col)) => Some(index(&t.chars, num(line)?, num(col)?)),
        None => Some(num(s)?.min(t.chars.len())),
    }
}

/// The range `a`..`b` of `t` as IUP writes it.
pub fn format_range(t: &Text, (a, b): (usize, usize)) -> String {
    format!("{}:{}", format_pos(t, a), format_pos(t, b))
}

/// The range `s` names in `t`: the two ends, in the order given;
/// `"NONE"` is the empty range at the caret, `"ALL"` the whole text.
pub fn parse_range(t: &Text, s: &str) -> Option<(usize, usize)> {
    match s.trim() {
        "NONE" | "" => Some((t.caret, t.caret)),
        "ALL" => Some((0, t.chars.len())),
        s => {
            let (a, b) = s.split_once(':')?;
            Some((parse_pos(t, a)?, parse_pos(t, b)?))
        }
    }
}
