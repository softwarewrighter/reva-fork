//! The command line: options, usage, exit codes.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use reva_vm::version;

/// What `--help` prints.
pub const USAGE: &str = "\
usage: reva [options] [file ...]

  -e, --eval TEXT      evaluate TEXT after the files (repeatable)
      --boot PATH      boot from PATH instead of the embedded reva.f
      --no-boot        start from the bare core (no reva.f at all)
      --image PATH     start from a saved image instead of booting
      --save-image PATH  after files and -e, save an image and exit
      --show-boot      print what booting printed (progress digits, errors)
      --trace          trace every VM step to stderr (very slow)
      --allow-shell    let `shell` (C `system`) run OS commands; off by default
      --no-edit        at a terminal, read plain lines (no history or line editing)
      --color WHEN     colour in `pp`/`hl-type` listings: auto (default), always, never
  -h, --help           this text
  -V, --version        version
";

/// The command line, parsed.
#[derive(Default)]
pub struct Options {
    pub files: Vec<String>,
    pub evals: Vec<String>,
    pub boot: Option<String>,
    pub no_boot: bool,
    pub image: Option<String>,
    pub save_image: Option<String>,
    pub show_boot: bool,
    pub trace: bool,
    pub allow_shell: bool,
    pub no_edit: bool,
    pub color: Option<bool>,
}

pub fn parse_args() -> Result<Options, String> {
    let mut o = Options::default();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut value = |name: &str| args.next().ok_or(format!("{name} needs a value"));
        match a.as_str() {
            "-e" | "--eval" => o.evals.push(value(&a)?),
            "--boot" => o.boot = Some(value(&a)?),
            "--no-boot" => o.no_boot = true,
            "--image" => o.image = Some(value(&a)?),
            "--save-image" => o.save_image = Some(value(&a)?),
            "--show-boot" => o.show_boot = true,
            "--trace" => o.trace = true,
            "--allow-shell" => o.allow_shell = true,
            "--no-edit" => o.no_edit = true,
            "--color" => {
                o.color = match value(&a)?.as_str() {
                    "auto" => None,
                    "always" => Some(true),
                    "never" => Some(false),
                    w => return Err(format!("--color {w}: expected auto, always or never")),
                }
            }
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                print!("{}", version::info());
                std::process::exit(0);
            }
            s if s.starts_with('-') && s.len() > 1 => return Err(format!("unknown option {s}")),
            _ => o.files.push(a),
        }
    }
    Ok(o)
}

/// `--color auto`: stdout is a terminal that takes ANSI escapes, and
/// `NO_COLOR` (no-color.org) is unset.  On Windows only where the console is
/// known to take them (Windows Terminal, or a `TERM` such as mintty's).
pub fn color_auto() -> bool {
    let env = |n: &str| std::env::var(n).ok().filter(|v| !v.is_empty());
    let ansi = if cfg!(windows) {
        env("WT_SESSION").is_some() || env("TERM").is_some_and(|t| t != "dumb")
    } else {
        env("TERM").is_none_or(|t| t != "dumb")
    };
    io::stdout().is_terminal() && env("NO_COLOR").is_none() && ansi
}

pub fn exit_code(code: i32) -> ExitCode {
    ExitCode::from((code & 0xff) as u8)
}
