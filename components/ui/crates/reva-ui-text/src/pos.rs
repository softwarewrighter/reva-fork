//! Lines and columns of a text, counted from 0; pure functions.

use crate::text::Motion;

/// The line and column of position `i` (clamped to the text).
pub fn line_col(chars: &[char], i: usize) -> (usize, usize) {
    let i = i.min(chars.len());
    let line = chars[..i].iter().filter(|c| **c == '\n').count();
    let start = chars[..i]
        .iter()
        .rposition(|c| *c == '\n')
        .map_or(0, |p| p + 1);
    (line, i - start)
}

/// Where line `line` starts and ends (its newline not included); past the
/// last line, the end of the text.
pub fn line_bounds(chars: &[char], line: usize) -> (usize, usize) {
    let mut start = 0;
    for _ in 0..line {
        match chars[start..].iter().position(|c| *c == '\n') {
            Some(p) => start += p + 1,
            None => return (chars.len(), chars.len()),
        }
    }
    let end = chars[start..]
        .iter()
        .position(|c| *c == '\n')
        .map_or(chars.len(), |p| start + p);
    (start, end)
}

/// The position of `line`, `col`, the column clamped to the line.
pub fn index(chars: &[char], line: usize, col: usize) -> usize {
    let (start, end) = line_bounds(chars, line);
    (start + col).min(end)
}

/// The number of lines (an empty text has one).
pub fn line_count(chars: &[char]) -> usize {
    chars.iter().filter(|c| **c == '\n').count() + 1
}

/// Where the caret at `caret` goes for `motion`.
pub fn moved(chars: &[char], caret: usize, motion: Motion) -> usize {
    let (line, col) = line_col(chars, caret);
    match motion {
        Motion::Left => caret.saturating_sub(1),
        Motion::Right => (caret + 1).min(chars.len()),
        Motion::Up if line == 0 => 0,
        Motion::Up => index(chars, line - 1, col),
        Motion::Down if line + 1 == line_count(chars) => chars.len(),
        Motion::Down => index(chars, line + 1, col),
        Motion::LineStart => line_bounds(chars, line).0,
        Motion::LineEnd => line_bounds(chars, line).1,
        Motion::TextStart => 0,
        Motion::TextEnd => chars.len(),
    }
}
