//! Booting a machine quietly, queueing the files and `-e` texts, and
//! `restart`.

use std::io::{self, Write};
use std::process::ExitCode;

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
        if !self.interactive {
            vm.quiet_prompt();
        } else if normal && !self.opts.show_boot {
            // The banner and first `ok>` prompt, as Reva showed them.
            host.write(rest.as_bytes());
        }
        vm.set_trace(self.opts.trace);
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
            match vm.run_until_blocked(host) {
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
}
