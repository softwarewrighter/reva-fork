//! Where the user's folder is, and the package files seeded into it.

use reva_desktop_pkg::manifest::{Manifest, package_file};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// The package an installer shipped: `package/` in the app's resources,
/// when it holds a `reva-package.json`.
pub fn installed_package(resources: Option<PathBuf>) -> Option<PathBuf> {
    let dir = resources?.join("package");
    dir.join("reva-package.json").is_file().then_some(dir)
}

/// The user's folder for `package`, made if missing and seeded (see
/// [`user_dir`] and [`seed`]); `None`: the package's own folder.
/// Documents is `~/Documents` on macOS, the XDG documents folder on Linux
/// (else `~/Documents`), the user's Documents folder on Windows;
/// `REVA_DOCUMENTS` replaces it (tests/check-installed.sh: a check never
/// touches a user's real folder).
pub fn open_user(
    explicit: Option<&Path>,
    installed: bool,
    package: &Path,
    m: &Manifest,
    notes: &mut Vec<String>,
) -> Result<Option<PathBuf>, String> {
    let docs = std::env::var_os("REVA_DOCUMENTS")
        .map(PathBuf::from)
        .or_else(dirs::document_dir)
        .or_else(|| dirs::home_dir().map(|h| h.join("Documents")))
        .unwrap_or_else(|| PathBuf::from("Documents"));
    let Some(dir) = user_dir(explicit, installed, m, &docs, notes) else {
        return Ok(None);
    };
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let dir = fs::canonicalize(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    seed(package, &dir, &m.seed, notes);
    Ok(Some(dir))
}

/// The user's folder: `explicit` (`--user`), else, when installed, the
/// manifest's `userFolder` (a plain name; default the package's name)
/// under `documents`.  `None`: the package's own folder.
pub fn user_dir(
    explicit: Option<&Path>,
    installed: bool,
    m: &Manifest,
    documents: &Path,
    notes: &mut Vec<String>,
) -> Option<PathBuf> {
    if let Some(dir) = explicit {
        return Some(dir.to_path_buf());
    }
    if !installed {
        return None;
    }
    let name = m.user_folder.as_deref().filter(|n| plain_name(n));
    if let (None, Some(bad)) = (name, &m.user_folder) {
        notes.push(format!("userFolder {bad:?} is not a plain folder name"));
    }
    let name = name.or(m.name.as_deref().filter(|n| plain_name(n)));
    Some(documents.join(name.unwrap_or("Reva")))
}

/// Copy each `seed` file of the package into the user's folder, unless it
/// is there already: a user's file is never overwritten.
pub fn seed(package: &Path, user: &Path, seeds: &[String], notes: &mut Vec<String>) {
    for rel in seeds {
        let (Some(from), Some(to)) = (package_file(package, rel), package_file(user, rel)) else {
            notes.push(format!("seed {rel:?} is not a path inside the package"));
            continue;
        };
        if to.exists() {
            continue;
        }
        let copied = to.parent().map_or(Ok(()), fs::create_dir_all);
        if let Err(e) = copied.and_then(|_| fs::copy(&from, &to).map(|_| ())) {
            notes.push(format!("seed {rel}: {e}"));
        }
    }
}

/// One plain folder name: no separators, `.` or `..`.
fn plain_name(name: &str) -> bool {
    let mut parts = Path::new(name).components();
    matches!(parts.next(), Some(Component::Normal(_))) && parts.next().is_none()
}
