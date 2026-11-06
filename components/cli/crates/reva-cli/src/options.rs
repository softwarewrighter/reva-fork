//! The command line: options, usage, exit codes.

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use reva_drawings::drawings::{Mode, parse_mode};
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
      --underflow WHAT when a word takes more than the stack holds: warn (default:
                       say so once and go on, as Reva did; the stack is empty again at the
                       next prompt), fail (stop with an error), or fail-at-word (stop in
                       the word that took too much, with its callers; slower)
      --check-execute  stop an `execute` of what is not code, naming who executed it
      --allow-shell    let `shell` (C `system`) run OS commands; off by default
      --no-edit        at a terminal, read plain lines (no history or line editing)
      --color WHEN     colour in `pp`/`hl-type` listings: auto (default), always, never
      --draw WHEN      turtle drawings to new files turtle-NNN.svg/.png (never overwriting):
                       auto (default: at a terminal; PNG shown inline in iTerm2/WezTerm),
                       svg, png, off
      --max-cpu SECONDS end a run that computes this long without waiting for input
                       (*** time limit, exit 124; 0: no limit; default 30 with --ui-script)
      --ui-script FILE answer IUP programs' user interface from FILE (headless; prints
                       what is shown; docs/ui-model.md)
      --port-check [--fix DIR] FILE...  report Windows-only paths and commands in
                       Forth sources; --fix writes portable copies to DIR
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
    pub underflow_fail: bool,
    /// --underflow fail-at-word: check at every word's return too.
    pub underflow_at_word: bool,
    /// --check-execute: stop an execute of what is not code.
    pub check_execute: bool,
    /// --max-cpu: seconds a run may compute without waiting for input
    /// (0: no limit; default: none, 30 under --ui-script).
    pub max_cpu: Option<f64>,
    pub allow_shell: bool,
    pub no_edit: bool,
    pub color: Option<bool>,
    pub ui_script: Option<String>,
    pub draw: Mode,
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
            "--check-execute" => o.check_execute = true,
            "--no-edit" => o.no_edit = true,
            "--ui-script" => o.ui_script = Some(value(&a)?),
            "--underflow" | "--max-cpu" | "--draw" | "--color" => setting(&mut o, &a, &value(&a)?)?,
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

/// An option whose value is one of a few words or a number.
fn setting(o: &mut Options, name: &str, v: &str) -> Result<(), String> {
    match (name, v) {
        ("--underflow", "warn" | "fail" | "fail-at-word") => {
            o.underflow_fail = v != "warn";
            o.underflow_at_word = v == "fail-at-word";
        }
        ("--underflow", _) => {
            return Err(format!("--underflow {v}: use warn, fail or fail-at-word"));
        }
        ("--max-cpu", _) => {
            o.max_cpu = Some(
                v.parse()
                    .map_err(|_| format!("--max-cpu {v}: seconds, a number"))?,
            );
        }
        ("--draw", _) => o.draw = parse_mode(v)?,
        (_, "auto") => o.color = None,
        (_, "always" | "never") => o.color = Some(v == "always"),
        _ => return Err(format!("--color {v}: expected auto, always or never")),
    }
    Ok(())
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
