//! After the files and `-e` texts: save an image, or read piped input to
//! its end, or read lines at the terminal; the exit code.  After each run a
//! changed turtle drawing goes to a new file (reva-drawings).

use std::io::{self, IsTerminal, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use crate::options::exit_code;
use crate::run::Run;
use reva_drawings::after::Written;
use reva_drawings::drawings::Drawings;
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
    let mut d = Drawings::new(run.opts.draw, io::stdout().is_terminal());
    let status = run.until_blocked(vm, host);
    drawn(&mut d, vm, host);
    if let Some(code) = finish(status) {
        return code;
    }
    if let Some(path) = &run.opts.save_image {
        return save(vm, path);
    }
    if !io::stdin().is_terminal() {
        return batch(run, vm, host, &mut d);
    }
    interactive(run, vm, host, &mut d)
}

/// What a run printed, flushed; then its drawing, if it changed, to a new
/// file.
fn drawn(d: &mut Drawings, vm: &mut Machine, host: &mut StdHost) {
    let _ = host.out.flush();
    match d.after(vm.canvas(), Path::new(".")) {
        Ok(Some(w)) => report(&w, &host.tail),
        Ok(None) => {}
        Err(e) => eprintln!("reva: the drawing was not saved: {e}"),
    }
}

/// The file's name, and the image where the terminal shows it.  At a
/// terminal they go above the line under way (the prompt), which is
/// cleared and printed again after them.
fn report(w: &Written, tail: &[u8]) {
    if !io::stdout().is_terminal() {
        eprintln!("drawing: {}", w.name);
        return;
    }
    let tail = String::from_utf8_lossy(tail);
    let mut out = io::stdout();
    let _ = write!(out, "\r\x1b[Kdrawing: {}\n{}{tail}", w.name, w.inline);
    let _ = out.flush();
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
fn batch(run: &Run, vm: &mut Machine, host: &mut StdHost, d: &mut Drawings) -> ExitCode {
    let mut all = Vec::new();
    if let Err(e) = io::stdin().lock().read_to_end(&mut all) {
        eprintln!("reva: stdin: {e}");
    }
    vm.feed(&all);
    vm.close_input();
    let status = run.until_blocked(vm, host);
    drawn(d, vm, host);
    finish(status).unwrap_or(ExitCode::SUCCESS)
}

/// Interactive: one line at a time; reva.f prints the prompt.
fn interactive(run: &Run, vm: &mut Machine, host: &mut StdHost, d: &mut Drawings) -> ExitCode {
    loop {
        match host.read_line() {
            Input::Eof => vm.close_input(),
            Input::Line(l) => vm.feed(l.as_bytes()),
        }
        let status = run.until_blocked(vm, host);
        drawn(d, vm, host);
        if let Some(code) = finish(status) {
            return code;
        }
    }
}
