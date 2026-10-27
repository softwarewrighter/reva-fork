//! Host calls on the CLI: SQLite through the reva-sqlite crate, answered
//! at once (rust/DESIGN.md 3.11).  Database names are VM paths, mapped to
//! host paths as the file words map them.

use reva_sqlite::{Reply, Sqlite};
use reva_vm::hostcall::{HostAnswer, HostCall, HostReply};

use crate::fs::StdFs;

/// SQLite for this run: paths arrive already mapped (see [`answer`]).
pub fn new() -> Sqlite {
    Sqlite::new(Box::new(|p| Some(p.into())))
}

/// Answer `call` with `sql`, mapping an `open`'s database name through
/// `fs` (`:memory:` stays as it is).
pub fn answer(sql: &mut Sqlite, fs: &mut StdFs, call: &HostCall) -> HostAnswer {
    if call.service != "sqlite" {
        return HostAnswer::Unavailable(format!("{} is not available on the CLI", call.service));
    }
    let mut bytes = call.bytes.clone();
    if call.op == "open" {
        let name = String::from_utf8_lossy(&bytes[0]).into_owned();
        if name != ":memory:" {
            bytes[0] = fs
                .resolve(&name)
                .1
                .to_string_lossy()
                .into_owned()
                .into_bytes();
        }
    }
    match sql.call(call.op, &call.ints, &bytes) {
        Some(Reply::Int(v)) => HostAnswer::Reply(HostReply::Int(v)),
        Some(Reply::Bytes(b)) => HostAnswer::Reply(HostReply::Bytes(b)),
        None => HostAnswer::Unavailable(format!("reva-sqlite has no operation {}", call.op)),
    }
}
