//! Connections: open, close, exec, the last row id, error messages.

use rusqlite::Connection;

use crate::Sqlite;
use crate::codes::{ERROR, MISUSE, OK, code};

/// An open connection and its last error message.
pub(crate) struct Db {
    pub(crate) conn: Connection,
    pub(crate) errmsg: String,
}

impl Sqlite {
    /// Open (or create) a database; `:memory:` is a private in-memory one.
    /// Returns the handle, or 0 with the code in `rc` (a path the host
    /// refuses is ERROR).
    pub(crate) fn open(&mut self, name: &str) -> i64 {
        let conn = if name == ":memory:" {
            Connection::open_in_memory()
        } else {
            match (self.map)(name) {
                Some(path) => Connection::open(path),
                None => return self.set_rc(ERROR, 0),
            }
        };
        match conn {
            Ok(conn) => {
                let h = self.handle();
                let errmsg = "not an error".into();
                self.dbs.insert(h, Db { conn, errmsg });
                self.set_rc(OK, h)
            }
            Err(e) => self.set_rc(code(&e), 0),
        }
    }

    /// Close a connection and its statements and queries.
    pub(crate) fn close(&mut self, db: i64) -> i64 {
        if self.dbs.remove(&db).is_none() {
            return self.set_rc(MISUSE, MISUSE);
        }
        self.stmts.retain(|_, s| s.db != db);
        self.queries.retain(|_, q| q.db != db);
        self.set_rc(OK, OK)
    }

    /// Run SQL text (any number of statements) for its effects.
    pub(crate) fn exec(&mut self, db: i64, sql: &str) -> i64 {
        let r = match self.dbs.get(&db) {
            Some(d) => d.conn.execute_batch(sql),
            None => return self.set_rc(MISUSE, MISUSE),
        };
        self.result(db, r.map(|()| OK))
    }

    pub(crate) fn last_id(&mut self, db: i64) -> i64 {
        self.dbs.get(&db).map_or(0, |d| d.conn.last_insert_rowid())
    }

    pub(crate) fn errmsg(&self, db: i64) -> String {
        self.dbs
            .get(&db)
            .map_or_else(|| "bad database handle".into(), |d| d.errmsg.clone())
    }

    /// Record the outcome of an operation on `db`: the code in `rc` and,
    /// on failure, the message for `errmsg`.  Returns the value on success
    /// and the (nonzero) code on failure.
    pub(crate) fn result(&mut self, db: i64, r: rusqlite::Result<i64>) -> i64 {
        match r {
            Ok(v) => self.set_rc(OK, v),
            Err(e) => {
                if let Some(d) = self.dbs.get_mut(&db) {
                    d.errmsg = e.to_string();
                }
                let c = match code(&e) {
                    OK => ERROR,
                    c => c,
                };
                self.set_rc(c, c)
            }
        }
    }

    /// Keep `rc` for the `rc` operation and return `v`.
    pub(crate) fn set_rc(&mut self, rc: i64, v: i64) -> i64 {
        self.rc = rc;
        v
    }
}
