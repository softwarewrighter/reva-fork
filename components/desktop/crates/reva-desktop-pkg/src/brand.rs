//! How a package brands the app: title, icon, Linux app id.

use crate::manifest::{Manifest, package_file, valid_id};
use std::fs;
use std::path::Path;

/// An icon: RGBA pixels, width, height.
pub type Icon = (Vec<u8>, u32, u32);

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
/// On macOS the icon also replaces the Dock icon (see `set_dock_icon`).
///
/// What run time cannot change, and needs a separate bundle or entry: the
/// macOS Finder and Launchpad icon (from the `.app` bundle's `.icns`), the
/// Windows Explorer and pinned-taskbar icon (from the `.exe` resource or a
/// shortcut's icon), and the Linux launcher entry (a `.desktop` file with
/// `StartupWMClass=<id>`).  See docs/desktop.md, "Branding".
#[derive(Default)]
pub struct Branding {
    pub title: String,
    /// RGBA pixels, width, height.
    pub icon: Option<Icon>,
    /// The icon file as read, for the macOS Dock (NSImage decodes it).
    pub icon_png: Option<Vec<u8>>,
    pub id: Option<String>,
}

/// Decode a PNG to RGBA, with the `png` crate Tauri already builds on
/// (its own `image-png` feature would add four more crates).
pub fn decode_png(bytes: &[u8]) -> Result<Icon, String> {
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

/// The branding of the package in `dir` (none: Reva's own), with notes for
/// what was not usable.
pub fn branding(dir: Option<&Path>, m: &Manifest, notes: &mut Vec<String>) -> Branding {
    let title = match (&m.title, &m.name) {
        (Some(t), _) => t.clone(),
        (None, Some(n)) => format!("{n} on Reva"),
        (None, None) => "Reva Forth".to_string(),
    };
    let (icon, icon_png) = match (dir, &m.icon) {
        (Some(dir), Some(rel)) => brand_icon(dir, rel, notes),
        _ => (None, None),
    };
    let id = brand_id(m.id.as_deref(), notes);
    Branding {
        title,
        icon,
        icon_png,
        id,
    }
}

/// The package's icon `rel`: decoded, and the file as read.
fn brand_icon(dir: &Path, rel: &str, notes: &mut Vec<String>) -> (Option<Icon>, Option<Vec<u8>>) {
    match package_file(dir, rel).map(fs::read) {
        Some(Ok(b)) => match decode_png(&b) {
            Ok(img) => (Some(img), Some(b)),
            Err(e) => {
                notes.push(format!("icon {rel}: not a usable PNG ({e}); using Reva's"));
                (None, None)
            }
        },
        Some(Err(e)) => {
            notes.push(format!("icon {rel}: {e}; using Reva's"));
            (None, None)
        }
        None => {
            notes.push(format!("icon {rel} is not a path inside the package"));
            (None, None)
        }
    }
}

/// The package's Linux app id, if usable.
fn brand_id(id: Option<&str>, notes: &mut Vec<String>) -> Option<String> {
    match id {
        Some(id) if valid_id(id) => Some(id.to_string()),
        Some(id) => {
            notes.push(format!("id {id:?} ignored: use letters, digits, . - _"));
            None
        }
        None => None,
    }
}
