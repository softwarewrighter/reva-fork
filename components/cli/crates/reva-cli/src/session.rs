//! After the files and `-e` texts: save an image, or read piped input to
//! its end, or read lines at the terminal; the exit code.

use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

use crate::options::exit_code;
use crate::run::Run;
use reva_stdhost::host::StdHost;
use reva_term::reader::Input;
use reva_vm::prelude::Input as _;
use reva_vm::prelude::*;
use reva_vm::{Machine, Status};

/// The run: begin, run the files and `-e` texts, then what follows.
pub fn session(run: &Run, vm: &mut Machine, host: &mut StdHost) -> ExitCode {
    if let Err(code) = run.begin(vm, host) {
        return code;
    }
    let status = run.until_blocked(vm, host);
    let _ = host.out.flush();
    if let Some(code) = finish(status) {
        return code;
    }
    if let Some(path) = &run.opts.save_image {
        return save(vm, path);
    }
    if !io::stdin().is_terminal() {
        return batch(run, vm, host);
    }
    interactive(run, vm, host)
}

/// The exit code once the machine halted (or a restart failed); `None`
/// while it waits for input.
fn finish(status: Result<Status, ExitCode>) -> Option<ExitCode> {
    match status {
        Ok(Status::Halted(code)) => Some(exit_code(code)),
        Err(code) => Some(code),
        Ok(_) => None,
    }
}

/// `--save-image PATH`: write the image and exit.
fn save(vm: &Machine, path: &str) -> ExitCode {
    if let Err(e) = std::fs::write(path, vm.save_image()) {
        eprintln!("reva: {path}: {e}");
        return ExitCode::from(2);
    }
    ExitCode::SUCCESS
}

/// Piped input: all of it, then end of input.
fn batch(run: &Run, vm: &mut Machine, host: &mut StdHost) -> ExitCode {
    let mut all = Vec::new();
    if let Err(e) = io::stdin().lock().read_to_end(&mut all) {
        eprintln!("reva: stdin: {e}");
    }
    vm.feed(&all);
    vm.close_input();
    let status = run.until_blocked(vm, host);
    let _ = host.out.flush();
    finish(status).unwrap_or(ExitCode::SUCCESS)
}

/// Interactive: one line at a time; reva.f prints the prompt.
fn interactive(run: &Run, vm: &mut Machine, host: &mut StdHost) -> ExitCode {
    loop {
        match host.read_line() {
            Input::Eof => vm.close_input(),
            Input::Line(l) => vm.feed(l.as_bytes()),
        }
        let status = run.until_blocked(vm, host);
        let _ = host.out.flush();
        if let Some(code) = finish(status) {
            return code;
        }
    }
}
