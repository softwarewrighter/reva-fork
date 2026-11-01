//! More windows, links in the browser, the Dock icon.

use reva_desktop_pkg::paths::web_url;

/// Windows `open_window` may open: each is `<name>.html` from the page.
const WINDOWS: &[(&str, f64, f64)] = &[("canvas", 820.0, 880.0)];

/// Open window `name` (or show it if it is open).
#[tauri::command]
pub fn open_window(app: tauri::AppHandle, name: String, title: String) -> Result<(), String> {
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

/// Open an http(s) link (the About dialog's) in the system browser.  Any
/// other scheme is refused, so the page cannot launch local programs.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    if !web_url(&url) {
        return Err(format!("not an http(s) link: {url}"));
    }
    let mut c = if cfg!(target_os = "macos") {
        std::process::Command::new("open")
    } else if cfg!(windows) {
        std::process::Command::new("explorer")
    } else {
        std::process::Command::new("xdg-open")
    };
    c.arg(&url)
        .spawn()
        .map(|mut child| drop(std::thread::spawn(move || child.wait())))
        .map_err(|e| format!("{url}: {e}"))
}

/// Show `png` as the app's Dock and app-switcher icon.  macOS takes these
/// from the running bundle, not from windows, so `set_icon` alone leaves
/// Reva's; NSApplication's `applicationIconImage` replaces it until the app
/// quits.  Finder and Launchpad still show the bundle's own icon.
#[cfg(target_os = "macos")]
pub fn set_dock_icon(png: &[u8]) {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let data = NSData::with_bytes(png);
    if let Some(img) = NSImage::initWithData(NSImage::alloc(), &data) {
        // SAFETY: on the main thread (mtm), with a non-nil image.
        unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&img)) };
    }
}
