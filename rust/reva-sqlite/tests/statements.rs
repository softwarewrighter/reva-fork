//! Statements: prepare, bind, step, columns, errors.

use reva_sqlite::*;

mod common;
use common::{bytes, int, mem, sql};

#[test]
fn exec_prepare_bind_step_column() {
    let (mut s, db) = mem();
    assert_eq!(
        int(s.call("exec", &[db], &sql("create table t(n integer, s text)"))),
        OK
    );
    let ins = int(s.call("prepare", &[db], &sql("insert into t values(?, ?)")));
    assert!(ins > 0);
    for (n, t) in [(1, "one"), (2, "two")] {
        assert_eq!(int(s.call("bind_int", &[ins, 1, n], &[])), OK);
        assert_eq!(int(s.call("bind_text", &[ins, 2], &sql(t))), OK);
        assert_eq!(int(s.call("step", &[ins], &[])), DONE);
        assert_eq!(int(s.call("reset", &[ins], &[])), OK);
    }
    assert_eq!(int(s.call("lastid", &[db], &[])), 2);
    let sel = int(s.call("prepare", &[db], &sql("select n, s from t order by n")));
    let mut got = Vec::new();
    while int(s.call("step", &[sel], &[])) == ROW {
        let n = int(s.call("column_int", &[sel, 0], &[]));
        let t = bytes(s.call("column_text", &[sel, 1], &[]));
        got.push((n, String::from_utf8(t).unwrap()));
    }
    assert_eq!(got, [(1, "one".into()), (2, "two".into())]);
    assert_eq!(int(s.call("finalize", &[sel], &[])), OK);
    assert_eq!(int(s.call("step", &[sel], &[])), MISUSE);
}

#[test]
fn errors_set_rc_and_errmsg() {
    let (mut s, db) = mem();
    assert_eq!(int(s.call("prepare", &[db], &sql("selec nonsense"))), 0);
    assert_eq!(int(s.call("rc", &[], &[])), ERROR);
    let msg = String::from_utf8(bytes(s.call("errmsg", &[db], &[]))).unwrap();
    assert!(msg.contains("syntax error"), "{msg}");
    assert_eq!(
        int(s.call("exec", &[db], &sql("insert into nowhere values(1)"))),
        ERROR
    );
    assert_eq!(int(s.call("close", &[db], &[])), OK);
    assert_eq!(int(s.call("close", &[db], &[])), MISUSE);
}

#[test]
fn zeroblob_blob_and_version() {
    let (mut s, db) = mem();
    s.call("exec", &[db], &sql("create table b(x)"));
    let ins = int(s.call("prepare", &[db], &sql("insert into b values(?)")));
    s.call("bind_zeroblob", &[ins, 1, 3], &[]);
    s.call("step", &[ins], &[]);
    let sel = int(s.call("prepare", &[db], &sql("select x from b")));
    assert_eq!(int(s.call("step", &[sel], &[])), ROW);
    assert_eq!(bytes(s.call("column_blob", &[sel, 0], &[])), [0, 0, 0]);
    assert!(bytes(s.call("version", &[], &[])).starts_with(b"3."));
    assert_eq!(s.call("nosuchop", &[], &[]), None);
}
