//! Row queries, for `sql_fetch` (what `sqlite3_exec` with a callback was):
//! run SQL text, then walk its rows from Forth one at a time, so no C
//! callback is involved.

use std::collections::VecDeque;

use rusqlite::Batch;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::types::Value;

use crate::Sqlite;
use crate::codes::{MISUSE, OK};
use crate::step::value_bytes;

/// The rows of a query not yet walked, and the current one.
pub(crate) struct Query {
    pub(crate) db: i64,
    rows: VecDeque<Vec<Value>>,
    row: Vec<Value>,
}

impl Sqlite {
    /// Run every statement in `sql` and keep the rows they return.
    /// Returns the query handle, or 0 with the code in `rc`.
    pub(crate) fn query(&mut self, db: i64, sql: &str) -> i64 {
        let r = match self.dbs.get(&db) {
            Some(d) => collect(&d.conn, sql),
            None => return self.set_rc(MISUSE, 0),
        };
        match r {
            Ok(rows) => {
                let h = self.handle();
                let row = Vec::new();
                self.queries.insert(h, Query { db, rows, row });
                self.set_rc(OK, h)
            }
            Err(e) => {
                self.result(db, Err(e));
                0
            }
        }
    }

    /// Move to the next row: its number of columns, or -1 after the last.
    pub(crate) fn next_row(&mut self, q: i64) -> i64 {
        let Some(query) = self.queries.get_mut(&q) else {
            return -1;
        };
        match query.rows.pop_front() {
            Some(row) => {
                query.row = row;
                query.row.len() as i64
            }
            None => -1,
        }
    }

    /// Column `col` (from 0) of the current row as text.
    pub(crate) fn getcol(&self, q: i64, col: i64) -> Vec<u8> {
        let value = self
            .queries
            .get(&q)
            .and_then(|query| query.row.get(usize::try_from(col).ok()?).cloned());
        value_bytes(value)
    }

    pub(crate) fn end_query(&mut self, q: i64) -> i64 {
        self.queries.remove(&q);
        self.set_rc(OK, OK)
    }
}

/// Every statement of `sql`, in order, and the rows they return.
fn collect(conn: &rusqlite::Connection, sql: &str) -> rusqlite::Result<VecDeque<Vec<Value>>> {
    let mut out = VecDeque::new();
    let mut batch = Batch::new(conn, sql);
    while let Some(mut stmt) = batch.next()? {
        let n = stmt.column_count();
        let mut rows = stmt.raw_query();
        while let Some(r) = rows.next()? {
            out.push_back(
                (0..n)
                    .map(|c| r.get::<_, Value>(c))
                    .collect::<Result<_, _>>()?,
            );
        }
    }
    Ok(out)
}
