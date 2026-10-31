//! No raw mode: plain lines.

pub struct Raw;
impl Raw {
    pub fn enter() -> Option<Raw> {
        None
    }
}
pub fn width() -> Option<usize> {
    None
}
