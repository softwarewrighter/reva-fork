//! Row queries (sql_fetch) and opening databases.

use reva_sqlite::*;

mod common;
use common::{bytes, int, mem, sql};

#[test]
fn query_walks_rows_of_every_statement() {
    let (mut s, db) = mem();
    s.call(
        "exec",
        &[db],
        &sql("create table t(a, b); insert into t values(1, null), (2, 'x');"),
    );
    let q = int(s.call("query", &[db], &sql("select a, b from t; select 'z';")));
    assert!(q > 0);
    let mut seen = Vec::new();
    loop {
        let n = int(s.call("next", &[q], &[]));
        if n < 0 {
            break;
        }
        let cols: Vec<String> = (0..n)
            .map(|c| String::from_utf8(bytes(s.call("getcol", &[q, c], &[]))).unwrap())
            .collect();
        seen.push(cols.join(","));
    }
    assert_eq!(seen, ["1,", "2,x", "z"]);
    assert_eq!(int(s.call("endquery", &[q], &[])), OK);
}

#[test]
fn a_refused_path_does_not_open() {
    let mut s = Sqlite::new(Box::new(|_| None));
    assert_eq!(int(s.call("open", &[], &sql("/elsewhere.db"))), 0);
    assert_eq!(int(s.call("rc", &[], &[])), ERROR);
}
