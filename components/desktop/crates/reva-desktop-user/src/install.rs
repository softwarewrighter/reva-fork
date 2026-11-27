//! The package an installer shipped: where it is, and what `--about`
//! says of it.

use crate::user::{documents, user_dir};
use reva_desktop_pkg::manifest::Manifest;
use std::fs;
use std::path::{Path, PathBuf};

/// The package an installer shipped: `package/` in the app's resources,
/// when it holds a `reva-package.json`.
pub fn installed_package(resources: Option<PathBuf>) -> Option<PathBuf> {
    let dir = resources?.join("package");
    dir.join("reva-package.json").is_file().then_some(dir)
}

/// A macOS bundle's `Contents/Resources` for the program `exe`, found by
/// its real path (symlinks resolved); `None` outside a bundle.
pub fn bundle_resources(exe: &Path) -> Option<PathBuf> {
    let real = fs::canonicalize(exe).ok()?;
    let res = real.parent()?.parent()?.join("Resources");
    res.is_dir().then_some(res)
}

/// `--about`'s lines for a package: its name and version (`about.version`),
/// where it is, and the user's folder (made at start, not here).
pub fn about_package(package: &Path, installed: bool, explicit: Option<&Path>) -> String {
    let m: Manifest = fs::read(package.join("reva-package.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let version = m.about.as_ref().and_then(|a| a["version"].as_str());
    let user = user_dir(explicit, installed, &m, &documents(), &mut Vec::new());
    format!(
        "package: {} {}\npackage dir: {}{}\nuser folder: {}\n",
        m.name.as_deref().unwrap_or("(no name)"),
        version.unwrap_or("(no version)"),
        package.display(),
        if installed { " (installed)" } else { "" },
        user.map_or("(the package's own)".into(), |u| u.display().to_string()),
    )
}
