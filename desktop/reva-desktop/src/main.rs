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
//!   after Forth's `restart`).  The manifest may also brand the window:
//!   `title`, `icon` (a PNG in the package) and `id` (the Linux app id
//!   that matches a `.desktop` entry; see [`Branding`]).  CoSy is one,
//!   kept in its own repository; this app knows nothing about CoSy.
//! * **Batch use.**  `--eval TEXT` feeds lines after start and
//!   `--print-output` copies the VM's output to stdout, so the app can be
//!   smoke-tested without a person (`xvfb-run reva-desktop --eval ...`).
//!   `bye` closes the window with Forth's exit code.
//!
//! * **More windows.**  `open_window` opens a page of the app (`canvas`:
//!   turtle graphics) in a second native window; the pages talk through
//!   the events `reva-ui-down` (main → window) and `reva-ui-up` (window →
//!   main, including `closed` when it is closed).  See www/ui.js.
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
  --canvas WHERE   where turtle graphics go: window (default), pane, or split
                   (REPL and graphics side by side); overrides the manifest
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
    /// Window title; default `<name> on Reva`.
    title: Option<String>,
    /// PNG in the package, for the window and the taskbar.
    icon: Option<String>,
    /// Linux app id (Wayland app_id, X11 WM_CLASS).
    id: Option<String>,
    #[serde(default)]
    start: Vec<String>,
    /// Where graphics go: "window", "pane" or "split" (see www/split.js).
    canvas: Option<String>,
}

/// The `canvas` values the page knows.
const CANVAS: &[&str] = &["window", "pane", "split"];

/// How a package brands the app, applied at run time so one installed
/// `reva-desktop` can look like each package it opens:
///
/// * **title** of the window (and of the page header);
/// * **icon** of the window, which is also the taskbar icon on Windows and
///   on X11 desktops;
/// * **id**, set as the program name before GTK starts: on Linux it becomes
///   the Wayland `app_id` and the X11 `WM_CLASS`, which is how GNOME, KDE
///   and others find the package's `.desktop` file, and with it the dock
///   icon and name.  Elsewhere it is ignored.
///
/// What run time cannot change, and needs a separate bundle or entry: the
/// macOS dock and Finder icon (from the `.app` bundle's `.icns`), the
/// Windows Explorer and pinned-taskbar icon (from the `.exe` resource or a
/// shortcut's icon), and the Linux launcher entry (a `.desktop` file with
/// `StartupWMClass=<id>`).  See docs/desktop.md, "Branding".
#[derive(Default)]
struct Branding {
    title: String,
    /// RGBA pixels, width, height.
    icon: Option<(Vec<u8>, u32, u32)>,
    id: Option<String>,
}

/// A Linux app id: what GTK accepts as a program name and a `.desktop`
/// file can match, kept to letters, digits, `.`, `-` and `_`.
fn valid_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// The real file for `rel`, a relative path inside the package (no `..`).
fn package_file(dir: &Path, rel: &str) -> Option<PathBuf> {
    let rel = Path::new(rel);
    let plain = rel.components().all(|c| matches!(c, Component::Normal(_)));
    (plain && !rel.as_os_str().is_empty()).then(|| dir.join(rel))
}

/// Decode a PNG to RGBA, with the `png` crate Tauri already builds on
/// (its own `image-png` feature would add four more crates).
fn decode_png(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    use png::{ColorType, Transformations};
    let mut d = png::Decoder::new(std::io::Cursor::new(bytes));
    d.set_transformations(Transformations::normalize_to_color8() | Transformations::ALPHA);
    let mut r = d.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; r.output_buffer_size().ok_or("image too large")?];
    let info = r.next_frame(&mut buf).map_err(|e| e.to_string())?;
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        ColorType::Rgba => buf,
        ColorType::GrayscaleAlpha => buf
            .chunks_exact(2)
            .flat_map(|ga| [ga[0], ga[0], ga[0], ga[1]])
            .collect(),
        c => return Err(format!("unexpected color type {c:?}")),
    };
    Ok((rgba, info.width, info.height))
}

fn branding(dir: Option<&Path>, m: &Manifest, notes: &mut Vec<String>) -> Branding {
    let title = match (&m.title, &m.name) {
        (Some(t), _) => t.clone(),
        (None, Some(n)) => format!("{n} on Reva"),
        (None, None) => "Reva Forth".to_string(),
    };
    let icon = match (dir, &m.icon) {
        (Some(dir), Some(rel)) => match package_file(dir, rel).map(fs::read) {
            Some(Ok(b)) => match decode_png(&b) {
                Ok(img) => Some(img),
                Err(e) => {
                    notes.push(format!("icon {rel}: not a usable PNG ({e}); using Reva's"));
                    None
                }
            },
            Some(Err(e)) => {
                notes.push(format!("icon {rel}: {e}; using Reva's"));
                None
            }
            None => {
                notes.push(format!("icon {rel} is not a path inside the package"));
                None
            }
        },
        _ => None,
    };
    let id = match &m.id {
        Some(id) if valid_id(id) => Some(id.clone()),
        Some(id) => {
            notes.push(format!("id {id:?} ignored: use letters, digits, . - _"));
            None
        }
        None => None,
    };
    Branding { title, icon, id }
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
    /// Where graphics go (`--canvas`, else the manifest's `canvas`).
    canvas: Option<String>,
    /// Notes for the user (skipped files, a missing manifest, ...).
    notes: Vec<String>,
}

struct State {
    session: Session,
    root: PathBuf,
    brand: Branding,
}

#[derive(Default)]
struct Options {
    package: Option<PathBuf>,
    root: Option<PathBuf>,
    evals: Vec<String>,
    print_output: bool,
    canvas: Option<String>,
}

fn parse_args() -> Result<Options, String> {
    let mut o = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = |name: &str| args.next().ok_or(format!("{name} needs a value"));
        match a.as_str() {
            "--package" => o.package = Some(value(&a)?.into()),
            "--root" => o.root = Some(value(&a)?.into()),
            "--canvas" => {
                let v = value(&a)?;
                if !CANVAS.contains(&v.as_str()) {
                    return Err(format!("--canvas {v}: use one of {}", CANVAS.join(", ")));
                }
                o.canvas = Some(v);
            }
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
    let package = o.package.is_some().then_some(root.as_path());
    let brand = branding(package, &manifest, &mut notes);
    let canvas = match (o.canvas, manifest.canvas) {
        (Some(c), _) => Some(c),
        (None, Some(c)) if CANVAS.contains(&c.as_str()) => Some(c),
        (None, Some(c)) => {
            notes.push(format!(
                "canvas {c:?} ignored: use one of {}",
                CANVAS.join(", ")
            ));
            None
        }
        (None, None) => None,
    };
    Ok(State {
        session: Session {
            title: brand.title.clone(),
            root: root.display().to_string(),
            start,
            input: o.evals,
            print_output: o.print_output,
            canvas,
            notes,
        },
        root,
        brand,
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

/// Windows `open_window` may open: each is `<name>.html` from the page.
const WINDOWS: &[(&str, f64, f64)] = &[("canvas", 820.0, 880.0)];

/// Open window `name` (or show it if it is open).
#[tauri::command]
fn open_window(app: tauri::AppHandle, name: String, title: String) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
    let &(name, w, h) = WINDOWS
        .iter()
        .find(|(n, _, _)| *n == name)
        .ok_or(format!("no window named {name}"))?;
    if let Some(win) = app.get_webview_window(name) {
        let _ = win.show();
        let _ = win.set_focus();
        return Ok(());
    }
    // Placement is left to the window manager.
    WebviewWindowBuilder::new(&app, name, WebviewUrl::App(format!("{name}.html").into()))
        .title(title)
        .inner_size(w, h)
        .build()
        .map(drop)
        .map_err(|e| format!("{name}: {e}"))
}

#[tauri::command]
fn halt(app: tauri::AppHandle, code: i32) {
    app.exit(code);
}

fn main() -> ExitCode {
    let mut state = match parse_args().and_then(build_state) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("reva-desktop: {e}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    // Before GTK starts (Builder::build), or it keeps "reva-desktop".
    #[cfg(target_os = "linux")]
    if let Some(id) = &state.brand.id {
        glib::set_prgname(Some(id.as_str()));
        glib::set_application_name(&state.brand.title);
    }
    let title = state.brand.title.clone();
    let icon = state.brand.icon.take();
    let app = tauri::Builder::default()
        .manage(Mutex::new(state))
        .invoke_handler(tauri::generate_handler![
            session,
            write_file,
            remove_file,
            print,
            open_window,
            halt
        ])
        .on_window_event(|win, ev| {
            use tauri::{Emitter, Manager};
            if let tauri::WindowEvent::Destroyed = ev {
                let name = win.label().to_string();
                if name != "main" {
                    let msg = serde_json::json!({ "type": "closed", "window": name });
                    let _ = win.app_handle().emit_to("main", "reva-ui-up", msg);
                }
            }
        })
        // The window is made here, not in tauri.conf.json, so it opens with
        // the package's title and icon instead of showing Reva's first.
        .setup(move |app| {
            let mut w = tauri::WebviewWindowBuilder::new(app, "main", Default::default())
                .title(&title)
                .inner_size(960.0, 680.0);
            if let Some((rgba, width, height)) = icon {
                w = w.icon(tauri::image::Image::new_owned(rgba, width, height))?;
            }
            w.build()?;
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
    fn package_files_stay_in_the_package() {
        let dir = Path::new("/p");
        assert_eq!(
            package_file(dir, "icons/a.png"),
            Some(dir.join("icons/a.png"))
        );
        assert_eq!(package_file(dir, "../a.png"), None);
        assert_eq!(package_file(dir, "/etc/a.png"), None);
        assert_eq!(package_file(dir, ""), None);
    }

    #[test]
    fn branding_defaults_and_checks() {
        let mut notes = Vec::new();
        let m: Manifest = serde_json::from_str(r#"{"name": "CoSy"}"#).unwrap();
        let b = branding(None, &m, &mut notes);
        assert_eq!(b.title, "CoSy on Reva");
        let m: Manifest =
            serde_json::from_str(r#"{"name": "CoSy", "title": "CoSy", "id": "co sy"}"#).unwrap();
        let b = branding(None, &m, &mut notes);
        assert_eq!((b.title.as_str(), b.id), ("CoSy", None));
        assert_eq!(notes.len(), 1);
        assert!(valid_id("cosy") && valid_id("org.example.Co_Sy-2"));
        assert!(!valid_id("") && !valid_id("a/b") && !valid_id(&"x".repeat(65)));
    }

    #[test]
    fn icons_must_be_png_files_in_the_package() {
        let dir = std::env::temp_dir().join(format!("reva-desktop-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        // a 1x1 gray PNG with alpha, the smallest a real file can be
        let mut ok = Vec::new();
        let mut e = png::Encoder::new(&mut ok, 1, 1);
        e.set_color(png::ColorType::GrayscaleAlpha);
        e.write_header().unwrap().write_image_data(&[7, 9]).unwrap();
        assert_eq!(decode_png(&ok), Ok((vec![7, 7, 7, 9], 1, 1)));
        fs::write(dir.join("ok.png"), ok).unwrap();
        fs::write(dir.join("bad.png"), b"GIF89a").unwrap();
        let icon = |rel: &str, notes: &mut Vec<String>| {
            let m = Manifest {
                icon: Some(rel.into()),
                ..Default::default()
            };
            branding(Some(&dir), &m, notes).icon.is_some()
        };
        let mut notes = Vec::new();
        assert!(icon("ok.png", &mut notes) && notes.is_empty());
        assert!(!icon("bad.png", &mut notes) && !icon("gone.png", &mut notes));
        assert!(!icon("../ok.png", &mut notes));
        assert_eq!(notes.len(), 3);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn latin1_round_trips_every_byte() {
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(from_latin1(&to_latin1(&all)), Some(all));
        assert_eq!(from_latin1("\u{100}"), None);
    }
}
