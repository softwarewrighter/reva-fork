//! Embed the library tree `lib/` (feature `embed-lib`, on by default) so
//! `needs` works on hosts without the files, such as the browser.  See
//! `embedded::lib_file`.  Also record the build host, commit and time for
//! `version` (see `src/version.rs`).
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Trimmed standard output of a command, or `None` if it failed.
fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (out.status.success() && !s.is_empty()).then_some(s)
}

fn build_info() {
    // Re-run when HEAD moves (checkout) or its branch gets a new commit.
    for path in ["HEAD", "packed-refs"] {
        if let Some(p) = run("git", &["rev-parse", "--git-path", path]) {
            println!("cargo:rerun-if-changed={p}");
        }
    }
    if let Some(r) = run("git", &["symbolic-ref", "-q", "HEAD"]) {
        if let Some(p) = run("git", &["rev-parse", "--git-path", &r]) {
            println!("cargo:rerun-if-changed={p}");
        }
    }
    let host = run("hostname", &["-s"])
        .or_else(|| run("hostname", &[]))
        .or_else(|| std::env::var("COMPUTERNAME").ok());
    let commit = run("git", &["rev-parse", "--short", "HEAD"]);
    let time = run("date", &["+%Y-%m-%dT%H:%M:%S%z"]);
    for (name, value) in [
        ("REVA_BUILD_HOST", host),
        ("REVA_BUILD_COMMIT", commit),
        ("REVA_BUILD_TIMESTAMP", time),
    ] {
        println!(
            "cargo:rustc-env={name}={}",
            value.unwrap_or_else(|| "unknown".into())
        );
    }
}

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
    build_info();
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lib");
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
