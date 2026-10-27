//! # reva-sqlite — SQLite for the Reva hosts
//!
//! The CLI and the desktop app answer the VM's SQLite *host calls*
//! (rust/DESIGN.md 3.11) with this crate.  It speaks in operations named
//! after `lib/db/sqlite`'s virtual functions, with plain integers and byte
//! strings, and keeps connections, statements and row queries behind small
//! integer handles, so nothing that crosses into the VM is a pointer.
//!
//! It uses rusqlite's safe API only.  A statement keeps its SQL and bound
//! values; its first `step` runs it and buffers the rows, and later steps
//! hand them out.  So a SELECT's whole result is held in memory until it is
//! reset or finalized, which suits a REPL.
//!
//! Result codes are SQLite's: 0 OK, 1 ERROR, 21 MISUSE (a bad handle),
//! 100 ROW, 101 DONE.  The last code is kept for the `rc` operation, and
//! each connection keeps its last error message for `errmsg`.

mod codes;
mod db;
mod query;
mod step;
mod stmt;

use std::collections::HashMap;
use std::path::PathBuf;

pub use codes::{DONE, ERROR, MISUSE, OK, ROW};

/// An operation's answer: a number, or bytes (text or a blob).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reply {
    Int(i64),
    Bytes(Vec<u8>),
}

/// Maps a database name as the program wrote it to a host path (the CLI:
/// its VM-to-host path mapping), or refuses it.
pub type PathMap = Box<dyn Fn(&str) -> Option<PathBuf>>;

/// The open connections, statements and row queries of one VM.
pub struct Sqlite {
    map: PathMap,
    dbs: HashMap<i64, db::Db>,
    stmts: HashMap<i64, stmt::Stmt>,
    queries: HashMap<i64, query::Query>,
    next: i64,
    rc: i64,
}

impl Sqlite {
    pub fn new(map: PathMap) -> Self {
        Sqlite {
            map,
            dbs: HashMap::new(),
            stmts: HashMap::new(),
            queries: HashMap::new(),
            next: 1,
            rc: OK,
        }
    }

    /// Run `op` with its integer and byte-string arguments (in the order
    /// lib/db/sqlite passes them).  An unknown operation is `None`.
    pub fn call(&mut self, op: &str, ints: &[i64], bytes: &[Vec<u8>]) -> Option<Reply> {
        let i = |k: usize| ints.get(k).copied().unwrap_or(0);
        let b = |k: usize| bytes.get(k).map_or(&[][..], Vec::as_slice);
        let text = |k: usize| String::from_utf8_lossy(b(k)).into_owned();
        Some(match op {
            "rc" => Reply::Int(self.rc),
            "version" => Reply::Bytes(rusqlite::version().as_bytes().to_vec()),
            "open" => Reply::Int(self.open(&text(0))),
            "close" => Reply::Int(self.close(i(0))),
            "exec" => Reply::Int(self.exec(i(0), &text(0))),
            "lastid" => Reply::Int(self.last_id(i(0))),
            "errmsg" => Reply::Bytes(self.errmsg(i(0)).into_bytes()),
            "query" => Reply::Int(self.query(i(0), &text(0))),
            "next" => Reply::Int(self.next_row(i(0))),
            "getcol" => Reply::Bytes(self.getcol(i(0), i(1))),
            "endquery" => Reply::Int(self.end_query(i(0))),
            _ => return self.stmt_call(op, ints, bytes),
        })
    }

    fn stmt_call(&mut self, op: &str, ints: &[i64], bytes: &[Vec<u8>]) -> Option<Reply> {
        let i = |k: usize| ints.get(k).copied().unwrap_or(0);
        let b = |k: usize| bytes.get(k).cloned().unwrap_or_default();
        Some(match op {
            "prepare" => Reply::Int(self.prepare(i(0), &String::from_utf8_lossy(&b(0)))),
            "bind_null" => Reply::Int(self.bind(i(0), i(1), stmt::Param::Null)),
            "bind_int" => Reply::Int(self.bind(i(0), i(1), stmt::Param::Int(i(2)))),
            "bind_text" => Reply::Int(self.bind(i(0), i(1), stmt::Param::Text(b(0)))),
            "bind_blob" => Reply::Int(self.bind(i(0), i(1), stmt::Param::Blob(b(0)))),
            "bind_zeroblob" => Reply::Int(self.bind(i(0), i(1), stmt::Param::Zeros(i(2)))),
            "reset" => Reply::Int(self.reset(i(0))),
            "clear" => Reply::Int(self.clear(i(0))),
            "step" => Reply::Int(self.step(i(0))),
            "finalize" => Reply::Int(self.finalize(i(0))),
            "column_int" => Reply::Int(self.column_int(i(0), i(1))),
            "column_text" | "column_blob" => Reply::Bytes(self.column_bytes(i(0), i(1))),
            _ => return None,
        })
    }

    /// A fresh handle.
    fn handle(&mut self) -> i64 {
        self.next += 1;
        self.next - 1
    }
}
