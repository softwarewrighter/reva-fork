//! The page's commands: its start, files written and removed, output for
//! `--print-output`, and `bye`.

use reva_desktop_pkg::paths::{from_latin1, real_path};
use reva_desktop_pkg::snapshot::snapshot;
use reva_desktop_state::state::{Session, State};
use serde::Serialize;
use std::fs;
use std::sync::Mutex;

/// What `session` sends: the session and the folder's files.
#[derive(Serialize)]
pub struct Start {
    #[serde(flatten)]
    session: Session,
    files: Vec<(String, String)>,
}

/// The session and the folder's files, for the page to start with.
#[tauri::command]
pub fn session(state: tauri::State<'_, Mutex<State>>) -> Start {
    let st = state.lock().unwrap();
    let mut session = st.session.clone();
    let files = snapshot(&st.root, &mut session.notes);
    Start { session, files }
}

/// The saved image to start from, as raw bytes (an ArrayBuffer in the
/// page), when the session says there is one.
#[tauri::command]
pub fn image(state: tauri::State<'_, Mutex<State>>) -> Result<tauri::ipc::Response, String> {
    let st = state.lock().unwrap();
    let path = st.image.as_ref().ok_or("no image")?;
    let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(tauri::ipc::Response::new(bytes))
}

/// Write a file the VM wrote (under the folder only).
#[tauri::command]
pub fn write_file(
    state: tauri::State<'_, Mutex<State>>,
    path: String,
    data: String,
) -> Result<(), String> {
    let st = state.lock().unwrap();
    let Some(p) = real_path(&st.root, &path) else {
        return Ok(());
    };
    let bytes = from_latin1(&data).ok_or("file data is not bytes")?;
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{path}: {e}"))?;
    }
    fs::write(&p, bytes).map_err(|e| format!("{path}: {e}"))
}

/// Remove a file the VM removed (under the folder only).
#[tauri::command]
pub fn remove_file(state: tauri::State<'_, Mutex<State>>, path: String) -> Result<(), String> {
    let st = state.lock().unwrap();
    let Some(p) = real_path(&st.root, &path) else {
        return Ok(());
    };
    match fs::remove_file(&p) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(format!("{path}: {e}")),
        _ => Ok(()),
    }
}

/// Copy what Reva printed to stdout (`--print-output`).
#[tauri::command]
pub fn print(text: String) {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
}

/// `bye`: close the app with Forth's exit code.
#[tauri::command]
pub fn halt(app: tauri::AppHandle, code: i32) {
    app.exit(code);
}
