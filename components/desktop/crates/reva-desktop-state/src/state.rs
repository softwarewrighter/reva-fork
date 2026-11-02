//! The state the commands share, and the session the page starts from.

use crate::options::{Options, home};
use crate::sqlite::new_sqlite;
use reva_desktop_pkg::brand::{Branding, branding};
use reva_desktop_pkg::manifest::{CANVAS, Manifest, package_file};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// What the page needs to start, sent by `session`.
#[derive(Serialize, Clone)]
pub struct Session {
    pub title: String,
    pub root: String,
    /// `include` of each start file: fed after boot and again after `restart`.
    pub start: Vec<String>,
    /// The `--eval` lines, fed once after the start lines.
    pub input: Vec<String>,
    pub print_output: bool,
    /// Where graphics go (`--canvas`, else the manifest's `canvas`).
    pub canvas: Option<String>,
    /// With `--package`: `{ name, title, about, hint }` from the manifest,
    /// for the About dialog and the header.
    pub package: Option<serde_json::Value>,
    /// Start from a saved image (the `image` command sends it).
    pub image: bool,
    /// Notes for the user (skipped files, a missing manifest, ...).
    pub notes: Vec<String>,
}

/// What the commands share.
pub struct State {
    pub session: Session,
    pub root: PathBuf,
    pub brand: Branding,
    /// The saved image to start from (`--image`, else the manifest's).
    pub image: Option<PathBuf>,
    /// SQLite for `lib/db/sqlite` (host calls; see the app's sql.rs).
    pub sql: reva_sqlite::Sqlite,
}

/// The state for options `o`: the folder (a package, `--root`, or
/// `~/Reva`), its manifest, branding, start lines and canvas.
pub fn build_state(o: Options) -> Result<State, String> {
    let mut notes = Vec::new();
    let (root, mut manifest) = open_root(&o, &mut notes)?;
    let start = manifest
        .start
        .iter()
        .map(|f| format!("include {f}"))
        .collect();
    let package = o.package.is_some().then_some(root.as_path());
    let brand = branding(package, &manifest, &mut notes);
    let canvas = choose_canvas(o.canvas, manifest.canvas.take(), &mut notes);
    let image = choose_image(o.image, package, manifest.image.take(), &mut notes);
    Ok(State {
        session: Session {
            title: brand.title.clone(),
            root: root.display().to_string(),
            start,
            input: o.evals,
            print_output: o.print_output,
            canvas,
            package: package.map(|_| package_json(&manifest)),
            image: image.is_some(),
            notes,
        },
        sql: new_sqlite(root.clone()),
        root,
        brand,
        image,
    })
}

/// The app's folder and its manifest: the package's (with its
/// `reva-package.json`, if any), or `--root`, or `~/Reva`.
fn open_root(o: &Options, notes: &mut Vec<String>) -> Result<(PathBuf, Manifest), String> {
    let Some(dir) = &o.package else {
        let root = o.root.clone().unwrap_or_else(|| home().join("Reva"));
        fs::create_dir_all(&root).map_err(|e| format!("{}: {e}", root.display()))?;
        let root = fs::canonicalize(&root).map_err(|e| format!("{}: {e}", root.display()))?;
        return Ok((root, Manifest::default()));
    };
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
    Ok((dir, manifest))
}

/// Where graphics go: `--canvas`, else the manifest's if the page knows it.
fn choose_canvas(
    option: Option<String>,
    manifest: Option<String>,
    notes: &mut Vec<String>,
) -> Option<String> {
    match (option, manifest) {
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
    }
}

/// The image to start from: `--image`, else the manifest's file in the
/// package, if it is there.
fn choose_image(
    option: Option<PathBuf>,
    package: Option<&Path>,
    manifest: Option<String>,
    notes: &mut Vec<String>,
) -> Option<PathBuf> {
    let path = option.or_else(|| package_file(package?, &manifest?))?;
    if path.is_file() {
        return Some(path);
    }
    notes.push(format!(
        "image {} not found: booting reva.f",
        path.display()
    ));
    None
}

/// The package as the page's About dialog and header see it.
fn package_json(m: &Manifest) -> serde_json::Value {
    serde_json::json!({
        "name": m.name,
        "id": m.id,
        "title": m.title,
        "about": m.about,
        "hint": m.hint,
    })
}
