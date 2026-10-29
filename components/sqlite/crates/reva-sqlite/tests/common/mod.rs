//! Helpers shared by the integration tests: the operations as
//! lib/db/sqlite drives them, through the public `Sqlite::call`.

#![allow(dead_code)]

use reva_sqlite::{Reply, Sqlite};

pub fn mem() -> (Sqlite, i64) {
    let mut s = Sqlite::new(Box::new(|_| None));
    let db = int(s.call("open", &[], &[b":memory:".to_vec()]));
    assert!(db > 0);
    (s, db)
}

pub fn int(r: Option<Reply>) -> i64 {
    match r {
        Some(Reply::Int(v)) => v,
        r => panic!("not an int: {r:?}"),
    }
}

pub fn bytes(r: Option<Reply>) -> Vec<u8> {
    match r {
        Some(Reply::Bytes(b)) => b,
        r => panic!("not bytes: {r:?}"),
    }
}

pub fn sql(s: &str) -> Vec<Vec<u8>> {
    vec![s.as_bytes().to_vec()]
}
