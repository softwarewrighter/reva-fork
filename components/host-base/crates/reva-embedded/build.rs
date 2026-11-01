//! Embed the library tree `lib/` (feature `embed-lib`, on by default) so
//! `needs` works on hosts without the files, such as the browser.  See
//! `embedded::lib_file`.  With it, the help index `help.idx` built from
//! the documentation (helpgen.rs).
use std::fmt::Write as _;

#[path = "helpgen.rs"]
mod helpgen;
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

/// The help index of `src/help.txt` and the `lib/` files, written to
/// `out`; `help` reads it as `/reva/lib/help.idx` (lib/helper).
fn help_index(root: &Path, lib_files: &[(String, PathBuf)], out: &Path) {
    let read =
        |p: &Path| String::from_utf8_lossy(&std::fs::read(p).unwrap_or_default()).into_owned();
    let help = root.join("src/help.txt");
    println!("cargo:rerun-if-changed={}", help.display());
    let mut entries = helpgen::entries(&read(&help), "src/reva.f", true);
    for (rel, path) in lib_files {
        entries.extend(helpgen::entries(&read(path), &format!("lib/{rel}"), false));
    }
    std::fs::write(out, helpgen::index(&entries)).unwrap();
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let lib = root.join("lib");
    println!("cargo:rerun-if-changed={}", lib.display());
    let mut files = Vec::new();
    if std::env::var_os("CARGO_FEATURE_EMBED_LIB").is_some() {
        walk(&lib, "", &mut files);
        let idx = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("help.idx");
        help_index(&root, &files, &idx);
        files.push(("help.idx".to_string(), idx));
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
