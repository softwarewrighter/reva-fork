//! Embed a static site into the binary.
//!
//! * `REVA_SERVE_SITE`: the directory to embed (default: `../reva-web/www`,
//!   the browser REPL as it is in the source tree).  Use the tree that
//!   `scripts/make-web-zip.sh` writes, so the binary serves exactly what
//!   the uploadable zip holds.
//! * `REVA_SERVE_NAME`: the app's name, used in messages and to recognise
//!   a copy of itself already running (default: `reva`).
//! * `REVA_SERVE_PORT`: the preferred port (default: 8421).
//!
//! Writes `$OUT_DIR/site.rs`: `NAME`, `PORT` and `FILES`, a list of
//! (path, bytes) sorted by path, each an `include_bytes!` of the file.
//! Dot files are skipped.

use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    for v in ["REVA_SERVE_SITE", "REVA_SERVE_NAME", "REVA_SERVE_PORT"] {
        println!("cargo:rerun-if-env-changed={v}");
    }
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let site = std::env::var_os("REVA_SERVE_SITE")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest.join("../../../web/crates/reva-web/www"));
    let site = fs::canonicalize(&site)
        .unwrap_or_else(|e| panic!("REVA_SERVE_SITE {}: {e}", site.display()));
    let name = std::env::var("REVA_SERVE_NAME").unwrap_or_else(|_| "reva".into());
    let port: u16 = std::env::var("REVA_SERVE_PORT")
        .map(|p| p.parse().expect("REVA_SERVE_PORT: not a port number"))
        .unwrap_or(8421);

    let mut files = Vec::new();
    walk(&site, &site, &mut files);
    files.sort();
    if !files.iter().any(|(p, _)| p == "index.html") {
        println!("cargo:warning={}: no index.html", site.display());
    }
    if !files.iter().any(|(p, _)| p == "reva.wasm") {
        println!(
            "cargo:warning={}: no reva.wasm (run scripts/build-web.sh first)",
            site.display()
        );
    }
    let mut out = format!(
        "/// The app's name.\npub const NAME: &str = {name:?};\n\
         /// The preferred port.\npub const PORT: u16 = {port};\n\
         /// The embedded files, sorted by path.\npub static FILES: &[(&str, &[u8])] = &[\n"
    );
    for (rel, abs) in &files {
        out += &format!(
            "    ({rel:?}, include_bytes!({:?})),\n",
            abs.display().to_string()
        );
    }
    out += "];\n";
    let dest = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("site.rs");
    fs::write(dest, out).unwrap();
}

fn walk(root: &Path, dir: &Path, files: &mut Vec<(String, PathBuf)>) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut entries: Vec<_> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .flatten()
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let path = e.path();
        if e.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, files);
        } else {
            println!("cargo:rerun-if-changed={}", path.display());
            let rel = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            files.push((rel, path));
        }
    }
}
