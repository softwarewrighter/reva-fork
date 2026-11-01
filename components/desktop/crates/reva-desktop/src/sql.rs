//! Host calls from the page (docs/design.md 3.11): SQLite through the
//! reva-sqlite crate, with every database inside the app's folder.  The
//! VM runs in the page's worker, so its `lib/db/sqlite` calls arrive here
//! as the `host_call` command and go back as `{ int }`, `{ bytes }` or
//! `{ error }` (see components/web/crates/reva-web/www/reva.js, `Reva.resume`).

use std::sync::Mutex;

use reva_desktop_state::state::State;
use reva_sqlite::Reply;
use serde::Deserialize;
use serde_json::{Value, json};

/// A host call as the page sends it (`Reva.hostCall`).
#[derive(Deserialize)]
pub struct Call {
    pub service: String,
    pub op: String,
    pub ints: Vec<i64>,
    pub bytes: Vec<Vec<u8>>,
}

#[tauri::command]
pub fn host_call(state: tauri::State<'_, Mutex<State>>, call: Call) -> Value {
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
