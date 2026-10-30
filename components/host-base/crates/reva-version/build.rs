//! Record the build host, commit and time for `version` (see
//! `src/version.rs`).
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
    if let Some(r) = run("git", &["symbolic-ref", "-q", "HEAD"])
        && let Some(p) = run("git", &["rev-parse", "--git-path", &r])
    {
        println!("cargo:rerun-if-changed={p}");
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

fn main() {
    build_info();
}
