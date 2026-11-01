//! A package's manifest, and files inside the package.

use serde::Deserialize;
use std::path::{Component, Path, PathBuf};

#[derive(Deserialize, Default)]
/// A package's `reva-package.json`.
pub struct Manifest {
    pub name: Option<String>,
    /// Window title; default `<name> on Reva`.
    pub title: Option<String>,
    /// PNG in the package, for the window and the taskbar.
    pub icon: Option<String>,
    /// Linux app id (Wayland app_id, X11 WM_CLASS).
    pub id: Option<String>,
    #[serde(default)]
    pub start: Vec<String>,
    /// Where graphics go: "window", "pane" or "split" (see www/split.js).
    pub canvas: Option<String>,
    /// The About dialog's entry for the package (see www/about.js); passed
    /// to the page as is, which checks it.
    pub about: Option<serde_json::Value>,
    /// The page header's hint (`code` in backticks).
    pub hint: Option<String>,
}

/// The `canvas` values the page knows.
pub const CANVAS: &[&str] = &["window", "pane", "split"];

/// A Linux app id: what GTK accepts as a program name and a `.desktop`
/// file can match, kept to letters, digits, `.`, `-` and `_`.
pub fn valid_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// The real file for `rel`, a relative path inside the package (no `..`).
pub fn package_file(dir: &Path, rel: &str) -> Option<PathBuf> {
    let rel = Path::new(rel);
    let plain = rel.components().all(|c| matches!(c, Component::Normal(_)));
    (plain && !rel.as_os_str().is_empty()).then(|| dir.join(rel))
}
