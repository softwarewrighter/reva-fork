//! SQLite's result codes, and the code for a rusqlite error.

pub const OK: i64 = 0;
pub const ERROR: i64 = 1;
pub const MISUSE: i64 = 21;
pub const ROW: i64 = 100;
pub const DONE: i64 = 101;

/// The primary result code SQLite gave for `e`, or ERROR for errors that
/// did not come from SQLite itself.
pub fn code(e: &rusqlite::Error) -> i64 {
    match e {
        rusqlite::Error::SqliteFailure(f, _) => i64::from(f.extended_code & 0xff),
        _ => ERROR,
    }
}
