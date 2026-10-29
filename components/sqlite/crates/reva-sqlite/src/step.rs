//! Running a statement and reading its rows.

use std::collections::VecDeque;

use rusqlite::types::Value;

use crate::Sqlite;
use crate::codes::{DONE, MISUSE, OK, ROW};

impl Sqlite {
    /// The next row: ROW (columns readable), DONE, or an error code.  The
    /// first step after `prepare` or `reset` runs the statement with its
    /// bindings and keeps all its rows.
    pub(crate) fn step(&mut self, st: i64) -> i64 {
        let Some(s) = self.stmts.get(&st) else {
            return self.set_rc(MISUSE, MISUSE);
        };
        if s.rows.is_none() {
            let db = s.db;
            match self.run(st) {
                Ok(rows) => self.stmts.get_mut(&st).map_or((), |s| s.rows = Some(rows)),
                Err(e) => return self.result(db, Err(e)),
            }
        }
        let Some(s) = self.stmts.get_mut(&st) else {
            return self.set_rc(MISUSE, MISUSE);
        };
        match s.rows.as_mut().and_then(VecDeque::pop_front) {
            Some(row) => {
                s.row = row;
                self.set_rc(OK, ROW)
            }
            None => {
                s.row.clear();
                self.set_rc(OK, DONE)
            }
        }
    }

    /// Run statement `st` and collect its rows (none for an INSERT,
    /// UPDATE, CREATE ...).
    fn run(&self, st: i64) -> rusqlite::Result<VecDeque<Vec<Value>>> {
        let s = &self.stmts[&st];
        let d = self.dbs.get(&s.db).ok_or(rusqlite::Error::InvalidQuery)?;
        let mut q = d.conn.prepare(&s.sql)?;
        for (i, p) in s.params.iter().enumerate() {
            if let Some(p) = p {
                q.raw_bind_parameter(i + 1, p.value())?;
            }
        }
        let n = q.column_count();
        let mut rows = q.raw_query();
        let mut out = VecDeque::new();
        while let Some(r) = rows.next()? {
            out.push_back(
                (0..n)
                    .map(|c| r.get::<_, Value>(c))
                    .collect::<Result<_, _>>()?,
            );
        }
        Ok(out)
    }

    /// Column `col` (from 0) of the current row as an integer, as SQLite
    /// converts: text by its leading digits, NULL and blobs as 0.
    pub(crate) fn column_int(&self, st: i64, col: i64) -> i64 {
        match self.column(st, col) {
            Some(Value::Integer(v)) => v,
            Some(Value::Real(v)) => v as i64,
            Some(Value::Text(t)) => leading_int(&t),
            _ => 0,
        }
    }

    /// Column `col` as bytes: text and blobs as stored, numbers as text,
    /// NULL as nothing.
    pub(crate) fn column_bytes(&self, st: i64, col: i64) -> Vec<u8> {
        value_bytes(self.column(st, col))
    }

    fn column(&self, st: i64, col: i64) -> Option<Value> {
        let s = self.stmts.get(&st)?;
        s.row.get(usize::try_from(col).ok()?).cloned()
    }
}

/// A value as the bytes `column_text` gives.
pub(crate) fn value_bytes(v: Option<Value>) -> Vec<u8> {
    match v {
        Some(Value::Integer(i)) => i.to_string().into_bytes(),
        Some(Value::Real(r)) => r.to_string().into_bytes(),
        Some(Value::Text(t)) => t.into_bytes(),
        Some(Value::Blob(b)) => b,
        _ => Vec::new(),
    }
}

/// The integer at the start of `t` (after spaces), as SQLite reads text.
fn leading_int(t: &str) -> i64 {
    let t = t.trim_start();
    let end = t
        .char_indices()
        .find(|&(i, c)| !(c.is_ascii_digit() || (i == 0 && (c == '-' || c == '+'))))
        .map_or(t.len(), |(i, _)| i);
    t[..end].parse().unwrap_or(0)
}
