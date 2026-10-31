//! `reva` — command-line host for the portable Reva core.
//!
//! ```text
//! reva [options] [file ...]
//! ```
//!
//! Boots `reva.f` (embedded), includes each file, evaluates each `-e` text,
//! then reads standard input: line by line with a prompt on a terminal, or
//! to end of input otherwise (so `reva x.f < /dev/null` is a batch run).
//! The exit status is the `bye` code (0 at end of input).
//!
//! `restart` starts over the way this run started (boot or `--image`),
//! includes the files and evaluates the `-e` texts again, then goes on
//! reading the input that is left.
//!
//! At a terminal, the line is edited before Forth sees it: history
//! (`~/.reva_history`, or `$REVA_HISTORY`), Up/Down, Ctrl-R search and
//! Emacs keys (see reva-lineedit, reva-term and docs/repl.md).
//!
//! The parts: [`options`] (the command line), [`run`] (booting and
//! `restart`), [`session`] (what follows: an image to save, piped input, or
//! the interactive loop); the host is reva-stdhost's `StdHost`.

mod options;
mod run;
mod session;

use std::io::{self, IsTerminal};
use std::process::ExitCode;

use options::{Options, USAGE, color_auto, parse_args};
use reva_stdhost::host::StdHost;
use reva_term::reader::LineReader;
use reva_vm::{Machine, Start, embedded};
use run::Run;

fn main() -> ExitCode {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("reva: {e}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let (start, mut vm) = match start_machine(&opts) {
        Ok(sm) => sm,
        Err(code) => return code,
    };
    let interactive = io::stdin().is_terminal() && opts.save_image.is_none();
    let run = Run {
        opts: &opts,
        start,
        interactive,
    };
    let reader = interactive.then(|| LineReader::new(!opts.no_edit));
    let color = opts.color.unwrap_or_else(color_auto);
    let mut host = StdHost::new(opts.allow_shell, color, reader);
    session::session(&run, &mut vm, &mut host)
}

/// The machine to start with: from `--image`, or booted (`reva.f`, the
/// `--boot` file, or nothing with `--no-boot`); and how it was started,
/// for `restart`.
fn start_machine(opts: &Options) -> Result<(Start, Machine), ExitCode> {
    let boot_path = opts
        .boot
        .clone()
        .unwrap_or_else(|| embedded::BOOT_PATH.to_string());
    match &opts.image {
        Some(path) => {
            let loaded = std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|b| Start::image(b).map_err(|e| e.to_string()));
            match loaded {
                Ok(sm) => Ok(sm),
                Err(e) => {
                    eprintln!("reva: {path}: {e}");
                    Err(ExitCode::from(2))
                }
            }
        }
        None => {
            let mut args = vec!["reva".to_string()];
            if !opts.no_boot {
                args.push(boot_path);
            }
            let start = Start::Boot(args);
            let vm = start
                .machine()
                .expect("a boot start always makes a machine");
            Ok((start, vm))
        }
    }
}
