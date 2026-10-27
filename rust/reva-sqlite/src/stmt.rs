//! Statements: prepare, bind, reset, clear the bindings, finalize.  Running
//! them and reading their rows is in step.rs.

use std::collections::VecDeque;

use rusqlite::types::Value;

use crate::Sqlite;
use crate::codes::{MISUSE, OK};

/// SQLite's code for a parameter index out of range.
const RANGE: i64 = 25;

/// A value bound to a statement parameter.
pub(crate) enum Param {
    Null,
    Int(i64),
    Text(Vec<u8>),
    Blob(Vec<u8>),
    Zeros(i64),
}

impl Param {
    pub(crate) fn value(&self) -> Value {
        match self {
            Param::Null => Value::Null,
            Param::Int(v) => Value::Integer(*v),
            Param::Text(t) => Value::Text(String::from_utf8_lossy(t).into_owned()),
            Param::Blob(b) => Value::Blob(b.clone()),
            Param::Zeros(n) => Value::Blob(vec![0; usize::try_from(*n).unwrap_or(0)]),
        }
    }
}

/// A prepared statement: its SQL and bindings, and once it has run, the
/// rows not yet stepped to and the current row.
pub(crate) struct Stmt {
    pub(crate) db: i64,
    pub(crate) sql: String,
    pub(crate) params: Vec<Option<Param>>,
    pub(crate) rows: Option<VecDeque<Vec<Value>>>,
    pub(crate) row: Vec<Value>,
}

impl Sqlite {
    /// Check `sql` compiles (one statement) and keep it.  Returns the
    /// statement handle, or 0 with the code in `rc` and the message for
    /// `errmsg`.
    pub(crate) fn prepare(&mut self, db: i64, sql: &str) -> i64 {
        let checked = match self.dbs.get(&db) {
            Some(d) => d.conn.prepare(sql).map(drop),
            None => return self.set_rc(MISUSE, 0),
        };
        if let Err(e) = checked {
            self.result(db, Err(e));
            return 0;
        }
        let h = self.handle();
        let (sql, params, rows, row) = (sql.to_string(), Vec::new(), None, Vec::new());
        self.stmts.insert(
            h,
            Stmt {
                db,
                sql,
                params,
                rows,
                row,
            },
        );
        self.set_rc(OK, h)
    }

    /// Bind parameter `index` (from 1).  Like SQLite, a binding takes
    /// effect at the next run: call `reset` after a step.
    pub(crate) fn bind(&mut self, st: i64, index: i64, p: Param) -> i64 {
        let Some(s) = self.stmts.get_mut(&st) else {
            return self.set_rc(MISUSE, MISUSE);
        };
        let Some(i) = usize::try_from(index).ok().and_then(|i| i.checked_sub(1)) else {
            return self.set_rc(RANGE, RANGE);
        };
        if s.params.len() <= i {
            s.params.resize_with(i + 1, || None);
        }
        s.params[i] = Some(p);
        self.set_rc(OK, OK)
    }

    /// Back to before the first step; the bindings stay.
    pub(crate) fn reset(&mut self, st: i64) -> i64 {
        self.with_stmt(st, |s| {
            s.rows = None;
            s.row.clear();
        })
    }

    /// Unbind every parameter (they read as NULL).
    pub(crate) fn clear(&mut self, st: i64) -> i64 {
        self.with_stmt(st, |s| s.params.clear())
    }

    pub(crate) fn finalize(&mut self, st: i64) -> i64 {
        match self.stmts.remove(&st) {
            Some(_) => self.set_rc(OK, OK),
            None => self.set_rc(MISUSE, MISUSE),
        }
    }

    fn with_stmt(&mut self, st: i64, f: impl FnOnce(&mut Stmt)) -> i64 {
        match self.stmts.get_mut(&st) {
            Some(s) => {
                f(s);
                self.set_rc(OK, OK)
            }
            None => self.set_rc(MISUSE, MISUSE),
        }
    }
}
