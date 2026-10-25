//! # reva-desktop: the Reva REPL in a native window
//!
//! The window shows the same page as the browser build (`rust/reva-web/www`)
//! and runs the same WebAssembly VM in a Web Worker.  What the desktop adds
//! is a host for that page (`host.js`, served in place of the browser's):
//!
//! * **Files on disk.**  The page keeps files through a Web Storage object.
//!   Here that object is backed by a folder: every file under it is handed
//!   to the VM at start (VM path `/x/y` = `<root>/x/y`), and each file the
//!   VM writes or deletes is written back through the commands below.
//!   The root is `~/Reva`, or the package folder.
//! * **Packages.**  `--package DIR` opens a folder with a `reva-package.json`
//!   manifest (`name`, `start`: files to `include` at start, and again
//!   after Forth's `restart`).  CoSy is one,
//!   kept in its own repository; this app knows nothing about CoSy.
//! * **Batch use.**  `--eval TEXT` feeds lines after start and
//!   `--print-output` copies the VM's output to stdout, so the app can be
//!   smoke-tested without a person (`xvfb-run reva-desktop --eval ...`).
//!   `bye` closes the window with Forth's exit code.
//!
//! Why the VM stays in the page and not in this process: one VM build for
//! the browser and the desktop, so both behave the same.  The commands
//! below are the whole native surface, and every path they take is checked
//! to stay under the root.

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

const USAGE: &str = "\
usage: reva-desktop [options]

  --package DIR    open DIR (a folder with reva-package.json); its files are
                   the VM's files, and its start files are included
  --root DIR       keep files in DIR instead of ~/Reva (ignored with --package)
  -e, --eval TEXT  feed TEXT as a line after start (repeatable)
  --print-output   copy everything Reva prints to stdout (for scripts and tests)
  -h, --help       this text
  -V, --version    version
";

/// Files above this size are not loaded into the VM at start (it keeps all
/// files in memory); a note says which were skipped.
const MAX_FILE: u64 = 16 << 20;
/// Nor more than this in total.
const MAX_TOTAL: u64 = 256 << 20;
/// VM paths under this prefix are the built-in library, never written out.
const BUILTIN: &str = "/reva/";

#[derive(Deserialize, Default)]
struct Manifest {
    name: Option<String>,
    #[serde(default)]
    start: Vec<String>,
}

/// What the page needs to start, sent by `session`.
#[derive(Serialize, Clone)]
struct Session {
    title: String,
    root: String,
    /// `include` of each start file: fed after boot and again after `restart`.
    start: Vec<String>,
    /// The `--eval` lines, fed once after the start lines.
    input: Vec<String>,
    print_output: bool,
    /// Notes for the user (skipped files, a missing manifest, ...).
    notes: Vec<String>,
}

struct State {
    session: Session,
    root: PathBuf,
}

#[derive(Default)]
struct Options {
    package: Option<PathBuf>,
    root: Option<PathBuf>,
    evals: Vec<String>,
    print_output: bool,
}

fn parse_args() -> Result<Options, String> {
    let mut o = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = |name: &str| args.next().ok_or(format!("{name} needs a value"));
        match a.as_str() {
            "--package" => o.package = Some(value(&a)?.into()),
            "--root" => o.root = Some(value(&a)?.into()),
            "-e" | "--eval" => o.evals.push(value(&a)?),
            "--print-output" => o.print_output = true,
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("reva-desktop {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            // macOS passes -psn_... when launched from Finder
            s if s.starts_with("-psn_") => {}
            s => return Err(format!("unknown option {s}")),
        }
    }
    Ok(o)
}

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn build_state(o: Options) -> Result<State, String> {
    let mut notes = Vec::new();
    let (root, manifest) = match &o.package {
        Some(dir) => {
            let dir = fs::canonicalize(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            let mpath = dir.join("reva-package.json");
            let manifest = match fs::read(&mpath) {
                Ok(b) => serde_json::from_slice::<Manifest>(&b)
                    .map_err(|e| format!("{}: {e}", mpath.display()))?,
                Err(_) => {
                    notes.push(format!(
                        "no reva-package.json in {}; opening it as a folder",
                        dir.display()
                    ));
                    Manifest::default()
                }
            };
            (dir, manifest)
        }
        None => {
            let root = o.root.clone().unwrap_or_else(|| home().join("Reva"));
            fs::create_dir_all(&root).map_err(|e| format!("{}: {e}", root.display()))?;
            let root = fs::canonicalize(&root).map_err(|e| format!("{}: {e}", root.display()))?;
            (root, Manifest::default())
        }
    };
    let start = manifest
        .start
        .iter()
        .map(|f| format!("include {f}"))
        .collect();
    let title = match &manifest.name {
        Some(n) => format!("{n} on Reva"),
        None => "Reva Forth".to_string(),
    };
    Ok(State {
        session: Session {
            title,
            root: root.display().to_string(),
            start,
            input: o.evals,
            print_output: o.print_output,
            notes,
        },
        root,
    })
}

/// The real file for VM path `vm`, if it is a plain path under the root
/// (no `..`, not the built-in library).
fn real_path(root: &Path, vm: &str) -> Option<PathBuf> {
    if vm.starts_with(BUILTIN) || !vm.starts_with('/') {
        return None;
    }
    let rel = Path::new(&vm[1..]);
    if rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return None;
    }
    Some(root.join(rel))
}

/// Bytes as a string of chars U+0000..U+00FF, one per byte: what the page's
/// Web Storage format already uses, and cheap to carry in JSON.
fn to_latin1(b: &[u8]) -> String {
    b.iter().map(|&c| c as char).collect()
}

fn from_latin1(s: &str) -> Option<Vec<u8>> {
    s.chars().map(|c| u8::try_from(c as u32).ok()).collect()
}

/// Every file under `root`, as (VM path, bytes), within the size limits.
fn snapshot(root: &Path, notes: &mut Vec<String>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut total = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let p = e.path();
            let Ok(meta) = e.metadata() else { continue };
            let name = e.file_name();
            if name.to_string_lossy().starts_with('.') {
                continue; // .git and friends
            }
            if meta.is_dir() {
                stack.push(p);
                continue;
            }
            let Ok(rel) = p.strip_prefix(root) else {
                continue;
            };
            let vm = format!("/{}", rel.to_string_lossy().replace('\\', "/"));
            if meta.len() > MAX_FILE || total + meta.len() > MAX_TOTAL {
                notes.push(format!("not loaded (too large): {vm}"));
                continue;
            }
            if let Ok(bytes) = fs::read(&p) {
                total += bytes.len() as u64;
                out.push((vm, to_latin1(&bytes)));
            }
        }
    }
    out
}

#[derive(Serialize)]
struct Start {
    #[serde(flatten)]
    session: Session,
    files: Vec<(String, String)>,
}

#[tauri::command]
fn session(state: tauri::State<'_, Mutex<State>>) -> Start {
    let st = state.lock().unwrap();
    let mut session = st.session.clone();
    let files = snapshot(&st.root, &mut session.notes);
    Start { session, files }
}

#[tauri::command]
fn write_file(
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

#[tauri::command]
fn remove_file(state: tauri::State<'_, Mutex<State>>, path: String) -> Result<(), String> {
    let st = state.lock().unwrap();
    let Some(p) = real_path(&st.root, &path) else {
        return Ok(());
    };
    match fs::remove_file(&p) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(format!("{path}: {e}")),
        _ => Ok(()),
    }
}

#[tauri::command]
fn print(text: String) {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
}

#[tauri::command]
fn halt(app: tauri::AppHandle, code: i32) {
    app.exit(code);
}

fn main() -> ExitCode {
    let state = match parse_args().and_then(build_state) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("reva-desktop: {e}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let title = state.session.title.clone();
    let app = tauri::Builder::default()
        .manage(Mutex::new(state))
        .invoke_handler(tauri::generate_handler![
            session,
            write_file,
            remove_file,
            print,
            halt
        ])
        .setup(move |app| {
            use tauri::Manager;
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_title(&title);
            }
            Ok(())
        })
        .build(tauri::generate_context!());
    match app {
        Ok(app) => {
            let code = app.run_return(|_, _| {});
            ExitCode::from(code.clamp(0, 255) as u8)
        }
        Err(e) => {
            eprintln!("reva-desktop: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_stay_under_the_root() {
        let root = Path::new("/home/u/Reva");
        assert_eq!(real_path(root, "/a/b.f"), Some(root.join("a/b.f")));
        assert_eq!(real_path(root, "/../etc/passwd"), None);
        assert_eq!(real_path(root, "/a/../../x"), None);
        assert_eq!(real_path(root, "/reva/lib/x"), None);
        assert_eq!(real_path(root, "relative"), None);
    }

    #[test]
    fn latin1_round_trips_every_byte() {
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(from_latin1(&to_latin1(&all)), Some(all));
        assert_eq!(from_latin1("\u{100}"), None);
    }
}
