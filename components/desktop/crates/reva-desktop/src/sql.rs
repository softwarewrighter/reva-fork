//! Host calls from the page (docs/design.md 3.11): SQLite through the
//! reva-sqlite crate, with every database inside the app's folder.  The
//! VM runs in the page's worker, so its `lib/db/sqlite` calls arrive here
//! as the `host_call` command and go back as `{ int }`, `{ bytes }` or
//! `{ error }` (see components/web/crates/reva-web/www/reva.js, `Reva.resume`).

use std::path::PathBuf;
use std::sync::Mutex;

use reva_sqlite::{Reply, Sqlite};
use serde::Deserialize;
use serde_json::{Value, json};

/// A host call as the page sends it (`Reva.hostCall`).
#[derive(Deserialize)]
pub struct Call {
    service: String,
    op: String,
    ints: Vec<i64>,
    bytes: Vec<Vec<u8>>,
}

/// SQLite for the folder `root`.  A database name is a VM path (a name
/// without a leading `/` is taken from `/`), mapped into the folder as the
/// file words map paths; one that would leave the folder does not open.
pub fn new(root: PathBuf) -> Sqlite {
    Sqlite::new(Box::new(move |name| {
        let vm = if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{name}")
        };
        crate::real_path(&root, &vm)
    }))
}

#[tauri::command]
pub fn host_call(state: tauri::State<'_, Mutex<crate::State>>, call: Call) -> Value {
    if call.service != "sqlite" {
        return json!({ "error": format!("{} is not available in the desktop app", call.service) });
    }
    let mut st = state.lock().unwrap();
    match st.sql.call(&call.op, &call.ints, &call.bytes) {
        Some(Reply::Int(v)) => json!({ "int": v }),
        Some(Reply::Bytes(b)) => json!({ "bytes": b }),
        None => json!({ "error": format!("reva-sqlite has no operation {}", call.op) }),
    }
}
