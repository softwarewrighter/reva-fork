//! Booting a machine quietly, queueing the files and `-e` texts, and
//! `restart`.

use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use crate::options::{Options, exit_code};
use reva_stdhost::host::StdHost;
use reva_vm::host::Console;
use reva_vm::prelude::*;
use reva_vm::{Machine, Start, Status, embedded, version};

/// How this run starts, kept for `restart`.
pub struct Run<'a> {
    pub opts: &'a Options,
    pub start: Start,
    pub interactive: bool,
}

impl Run<'_> {
    /// Start `vm`: boot quietly (reva.f prints progress marks, then, as the
    /// restarted system, its banner and first prompt; anything else means
    /// the boot went wrong, so show it), then queue the files and `-e`
    /// texts.
    pub fn begin(&self, vm: &mut Machine, host: &mut StdHost) -> Result<(), ExitCode> {
        let boot_out = self.boot(vm, host)?;
        let (marks, rest) = embedded::split_boot_output(&boot_out);
        let normal =
            !rest.contains("***") && (rest.is_empty() || rest.starts_with(version::BANNER));
        if self.opts.show_boot || !normal {
            eprint!("{marks}{rest}");
            if !boot_out.ends_with('\n') && !boot_out.is_empty() {
                eprintln!();
            }
        }
        if !matches!(self.start, Start::Image(..)) {
            vm.record_prompt();
        }
        if !self.interactive {
            vm.quiet_prompt();
        } else if normal && !self.opts.show_boot {
            vm.restore_prompt();
            // The banner and first `ok>` prompt, as Reva showed them.
            host.write(rest.as_bytes());
        }
        vm.set_trace(self.opts.trace);
        vm.underflow_fail = self.opts.underflow_fail;
        self.queue(vm);
        Ok(())
    }

    /// Boot with the output captured: what booting printed, or the exit
    /// code if it halted or restarted.
    fn boot(&self, vm: &mut Machine, host: &mut StdHost) -> Result<String, ExitCode> {
        host.capture = Some(Vec::new());
        let status = vm.run_until_blocked(host);
        let boot_out = host.capture.take().unwrap_or_default();
        match status {
            Status::Halted(code) => {
                io::stdout().write_all(&boot_out).ok();
                eprintln!("reva: halted during boot");
                Err(exit_code(code))
            }
            Status::Restart => {
                io::stdout().write_all(&boot_out).ok();
                eprintln!("reva: restart during boot");
                Err(ExitCode::from(2))
            }
            _ => Ok(String::from_utf8_lossy(&boot_out).into_owned()),
        }
    }

    /// Queue `include FILE` for each file, then each `-e` text.
    fn queue(&self, vm: &mut Machine) {
        let mut pending = Vec::new();
        for f in &self.opts.files {
            pending.extend_from_slice(format!("include {f}\n").as_bytes());
        }
        for e in &self.opts.evals {
            pending.extend_from_slice(e.as_bytes());
            pending.push(b'\n');
        }
        vm.feed(&pending);
    }

    /// Run until `vm` needs input or halts; on `restart`, start a new
    /// machine the same way and hand it the input not yet read.
    pub fn until_blocked(&self, vm: &mut Machine, host: &mut StdHost) -> Result<Status, ExitCode> {
        loop {
            match self.run_limited(vm, host)? {
                Status::Restart => {
                    let (input, closed) = vm.take_input();
                    *vm = self.start.machine().map_err(|e| {
                        eprintln!("reva: restart: {e}");
                        ExitCode::from(2)
                    })?;
                    self.begin(vm, host)?;
                    vm.feed(&input);
                    if closed {
                        vm.close_input();
                    }
                }
                s => return Ok(s),
            }
        }
    }

    /// Run until `vm` blocks, or, past `--max-cpu` seconds without waiting
    /// for input (a runaway loop), stop with `*** time limit` (exit 124, as
    /// `timeout` does).
    fn run_limited(&self, vm: &mut Machine, host: &mut StdHost) -> Result<Status, ExitCode> {
        let limit = match (self.opts.max_cpu, &self.opts.ui_script) {
            (Some(s), _) if s > 0.0 => Some(Duration::from_secs_f64(s)),
            (None, Some(_)) => Some(Duration::from_secs(30)),
            _ => None,
        };
        let t0 = Instant::now();
        loop {
            match reva_vm::prelude::Run::run(&mut vm.0, host, 1 << 20) {
                Status::Yielded if limit.is_some_and(|l| t0.elapsed() > l) => {
                    let _ = host.out.flush();
                    let secs = limit.map_or(0.0, |l| l.as_secs_f64());
                    eprintln!("\n*** time limit: {secs} s without waiting for input (--max-cpu)");
                    return Err(ExitCode::from(124));
                }
                Status::Yielded => continue,
                s => return Ok(s),
            }
        }
    }
}
