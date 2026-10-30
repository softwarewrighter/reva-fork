//! Embed the library tree `lib/` (feature `embed-lib`, on by default) so
//! `needs` works on hosts without the files, such as the browser.  See
//! `embedded::lib_file`.
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let r = if rel.is_empty() {
            name.clone()
        } else {
            format!("{rel}/{name}")
        };
        let p = e.path();
        if p.is_dir() {
            walk(&p, &r, out);
        } else if p.is_file() {
            out.push((r, p));
        }
    }
}

fn main() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../lib");
    println!("cargo:rerun-if-changed={}", lib.display());
    let mut files = Vec::new();
    if std::env::var_os("CARGO_FEATURE_EMBED_LIB").is_some() {
        walk(&lib, "", &mut files);
    }
    files.sort();
    let mut src = String::from("pub static LIB_FILES: &[(&str, &[u8])] = &[\n");
    for (rel, path) in &files {
        let abs = path.canonicalize().unwrap_or_else(|_| path.clone());
        writeln!(
            src,
            "    ({rel:?}, include_bytes!({:?})),",
            abs.display().to_string()
        )
        .unwrap();
    }
    src.push_str("];\n");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("lib_files.rs");
    std::fs::write(out, src).unwrap();
}
