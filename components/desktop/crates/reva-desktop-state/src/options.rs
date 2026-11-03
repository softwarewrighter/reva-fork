//! The command line.

use reva_desktop_pkg::manifest::CANVAS;
use std::path::PathBuf;

/// What `--help` prints.
pub const USAGE: &str = "\
usage: reva-desktop [options]

  --package DIR    open DIR (a folder with reva-package.json); its files are
                   the VM's files, and its start files are included
  --root DIR       keep files in DIR instead of ~/Reva (ignored with --package)
  --canvas WHERE   where turtle graphics go: window (default), pane, or split
                   (REPL and graphics side by side); overrides the manifest
  --image FILE     start from a saved image (save NAME, reva --save-image)
                   instead of booting reva.f; overrides the manifest
  -e, --eval TEXT  feed TEXT as a line after start (repeatable)
  --print-output   copy everything Reva prints to stdout (for scripts and tests)
  -h, --help       this text
  -V, --version    version
";

#[derive(Default)]
/// The command line, parsed.
pub struct Options {
    pub package: Option<PathBuf>,
    pub root: Option<PathBuf>,
    pub image: Option<PathBuf>,
    pub evals: Vec<String>,
    pub print_output: bool,
    pub canvas: Option<String>,
}

/// Parse the command line (`--help` and `--version` exit here).
pub fn parse_args() -> Result<Options, String> {
    let mut o = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = |name: &str| args.next().ok_or(format!("{name} needs a value"));
        match a.as_str() {
            "--package" => o.package = Some(value(&a)?.into()),
            "--root" => o.root = Some(value(&a)?.into()),
            "--image" => o.image = Some(value(&a)?.into()),
            "--canvas" => {
                let v = value(&a)?;
                if !CANVAS.contains(&v.as_str()) {
                    return Err(format!("--canvas {v}: use one of {}", CANVAS.join(", ")));
                }
                o.canvas = Some(v);
            }
            "-e" | "--eval" => o.evals.push(value(&a)?),
            "--print-output" => o.print_output = true,
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!(
                    "reva-desktop {}",
                    include_str!("../../../../../VERSION").trim_ascii()
                );
                std::process::exit(0);
            }
            // macOS passes -psn_... when launched from Finder
            s if s.starts_with("-psn_") => {}
            s => return Err(format!("unknown option {s}")),
        }
    }
    Ok(o)
}

/// The home directory (`.` if unknown).
pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
