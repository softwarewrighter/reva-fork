//! # reva-desktop: the Reva REPL in a native window
//!
//! The window shows the same page as the browser build (`components/web/crates/reva-web/www`)
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
//!   that matches a `.desktop` entry; see reva-desktop-pkg's `Branding`).  CoSy is one,
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

use std::process::ExitCode;
use std::sync::Mutex;

use reva_desktop_state::options::{USAGE, parse_args};
use reva_desktop_state::state::{State, build_state};

mod commands;
mod sql;
mod windows;

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
    let icon = state.brand.icon.take();
    let dock_png = state.brand.icon_png.take();
    match build(state, icon) {
        Ok(app) => run(app, dock_png),
        Err(e) => {
            eprintln!("reva-desktop: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The app: the commands, the window events, and the main window, made
/// here (not in tauri.conf.json) so it opens with the package's title and
/// icon instead of showing Reva's first.
fn build(state: State, icon: Option<(Vec<u8>, u32, u32)>) -> tauri::Result<tauri::App> {
    let title = state.brand.title.clone();
    tauri::Builder::default()
        .manage(Mutex::new(state))
        .invoke_handler(tauri::generate_handler![
            commands::session,
            commands::image,
            commands::write_file,
            commands::remove_file,
            commands::print,
            windows::open_window,
            windows::open_url,
            commands::halt,
            sql::host_call
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
        .setup(move |app| {
            let mut w = tauri::WebviewWindowBuilder::new(app, "main", Default::default())
                .title(&title)
                .inner_size(1280.0, 860.0);
            if let Some((rgba, width, height)) = icon {
                w = w.icon(tauri::image::Image::new_owned(rgba, width, height))?;
            }
            w.build()?;
            Ok(())
        })
        .build(tauri::generate_context!())
}

/// Run until the last window closes or `bye`; the exit code.
fn run(
    app: tauri::App,
    #[cfg_attr(not(target_os = "macos"), allow(unused_variables))] dock_png: Option<Vec<u8>>,
) -> ExitCode {
    // On Ready, after macOS has finished launching and applied the bundle's
    // icon, which would otherwise replace ours.
    let code = app.run_return(move |_, ev| {
        #[cfg(target_os = "macos")]
        if let (tauri::RunEvent::Ready, Some(png)) = (&ev, &dock_png) {
            windows::set_dock_icon(png);
        }
        let _ = &ev;
    });
    ExitCode::from(code.clamp(0, 255) as u8)
}
