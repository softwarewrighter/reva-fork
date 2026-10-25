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

use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reva_vm::vfs::FileSystem;
use reva_vm::{embedded, Host, Machine, Start, Status};

const USAGE: &str = "\
usage: reva [options] [file ...]

  -e, --eval TEXT      evaluate TEXT after the files (repeatable)
      --boot PATH      boot from PATH instead of the embedded reva.f
      --no-boot        start from the bare core (no reva.f at all)
      --image PATH     start from a saved image instead of booting
      --save-image PATH  after files and -e, save an image and exit
      --show-boot      print what booting printed (progress digits, errors)
      --trace          trace every VM step to stderr (very slow)
  -h, --help           this text
  -V, --version        version
";

mod fs;

struct StdHost {
    out: io::BufWriter<io::Stdout>,
    fs: fs::StdFs,
    /// While booting: output is collected here so the progress digits
    /// stay quiet.
    capture: Option<Vec<u8>>,
}

/// Process environment, clock and file system, shared by both hosts.
macro_rules! os_services {
    () => {
        fn getenv(&mut self, name: &str) -> Option<String> {
            std::env::var(name).ok()
        }
        fn setenv(&mut self, name: &str, value: &str) {
            std::env::set_var(name, value);
        }
        fn now(&mut self) -> Option<(u64, u32)> {
            let d = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
            Some((d.as_secs(), d.subsec_micros()))
        }
        fn sleep_ms(&mut self, ms: u32) {
            std::thread::sleep(Duration::from_millis(ms.into()));
        }
        fn fs(&mut self) -> Option<&mut dyn FileSystem> {
            Some(&mut self.fs)
        }
    };
}

impl Host for StdHost {
    fn write(&mut self, bytes: &[u8]) {
        match &mut self.capture {
            Some(c) => c.extend_from_slice(bytes),
            None => {
                let _ = self.out.write_all(bytes);
            }
        }
    }

    os_services!();

    fn read_input_blocking(&mut self) -> Option<Vec<u8>> {
        if self.capture.is_some() {
            return None;
        }
        let _ = self.out.flush();
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(_) => Some(line.into_bytes()),
            Err(_) => None,
        }
    }
}

#[derive(Default)]
struct Options {
    files: Vec<String>,
    evals: Vec<String>,
    boot: Option<String>,
    no_boot: bool,
    image: Option<String>,
    save_image: Option<String>,
    show_boot: bool,
    trace: bool,
}

fn parse_args() -> Result<Options, String> {
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
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("reva {} (portable vx86 core)", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            s if s.starts_with('-') && s.len() > 1 => return Err(format!("unknown option {s}")),
            _ => o.files.push(a),
        }
    }
    Ok(o)
}

fn exit_code(code: i32) -> ExitCode {
    ExitCode::from((code & 0xff) as u8)
}

fn main() -> ExitCode {
    let opts = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("reva: {e}\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    let boot_path = opts
        .boot
        .clone()
        .unwrap_or_else(|| embedded::BOOT_PATH.to_string());
    let (start, mut vm) = match &opts.image {
        Some(path) => {
            let loaded = std::fs::read(path)
                .map_err(|e| e.to_string())
                .and_then(|b| Start::image(b).map_err(|e| e.to_string()));
            match loaded {
                Ok(sm) => sm,
                Err(e) => {
                    eprintln!("reva: {path}: {e}");
                    return ExitCode::from(2);
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
            (start, vm)
        }
    };
    let run = Run {
        interactive: io::stdin().is_terminal() && opts.save_image.is_none(),
        opts: &opts,
        start,
    };
    let mut host = StdHost {
        out: io::BufWriter::new(io::stdout()),
        fs: fs::StdFs::default(),
        capture: None,
    };
    if let Err(code) = run.begin(&mut vm, &mut host) {
        return code;
    }
    let status = run.until_blocked(&mut vm, &mut host);
    let _ = host.out.flush();
    match status {
        Ok(Status::Halted(code)) => return exit_code(code),
        Err(code) => return code,
        Ok(_) => {}
    }

    if let Some(path) = &opts.save_image {
        if let Err(e) = std::fs::write(path, vm.save_image()) {
            eprintln!("reva: {path}: {e}");
            return ExitCode::from(2);
        }
        return ExitCode::SUCCESS;
    }

    let stdin = io::stdin();
    if !stdin.is_terminal() {
        let mut all = Vec::new();
        if let Err(e) = stdin.lock().read_to_end(&mut all) {
            eprintln!("reva: stdin: {e}");
        }
        vm.feed(&all);
        vm.close_input();
        let status = run.until_blocked(&mut vm, &mut host);
        let _ = host.out.flush();
        return match status {
            Ok(Status::Halted(code)) => exit_code(code),
            Err(code) => code,
            Ok(_) => ExitCode::SUCCESS,
        };
    }

    // Interactive: one line at a time; reva.f prints the prompt.
    loop {
        let _ = host.out.flush();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => vm.close_input(),
            Ok(_) => vm.feed(line.as_bytes()),
        }
        let status = run.until_blocked(&mut vm, &mut host);
        let _ = host.out.flush();
        match status {
            Ok(Status::Halted(code)) => return exit_code(code),
            Err(code) => return code,
            Ok(_) => {}
        }
    }
}

/// How this run starts, kept for `restart`.
struct Run<'a> {
    opts: &'a Options,
    start: Start,
    interactive: bool,
}

impl Run<'_> {
    /// Start `vm`: boot quietly (reva.f prints progress marks, then, as the
    /// restarted system, its banner and first prompt; anything else means
    /// the boot went wrong, so show it), then queue the files and `-e`
    /// texts.
    fn begin(&self, vm: &mut Machine, host: &mut StdHost) -> Result<(), ExitCode> {
        let opts = self.opts;
        host.capture = Some(Vec::new());
        let status = vm.run_until_blocked(host);
        let boot_out = host.capture.take().unwrap_or_default();
        match status {
            Status::Halted(code) => {
                io::stdout().write_all(&boot_out).ok();
                eprintln!("reva: halted during boot");
                return Err(exit_code(code));
            }
            Status::Restart => {
                io::stdout().write_all(&boot_out).ok();
                eprintln!("reva: restart during boot");
                return Err(ExitCode::from(2));
            }
            _ => {}
        }
        let boot_out = String::from_utf8_lossy(&boot_out).into_owned();
        let (marks, rest) = embedded::split_boot_output(&boot_out);
        let normal = !rest.contains("***") && (rest.is_empty() || rest.starts_with("Reva "));
        if opts.show_boot || !normal {
            eprint!("{marks}{rest}");
            if !boot_out.ends_with('\n') && !boot_out.is_empty() {
                eprintln!();
            }
        }
        if self.interactive {
            // The banner and first `ok>` prompt, as Reva showed them.
            if normal && !opts.show_boot {
                let _ = host.out.write_all(rest.as_bytes());
            }
        } else {
            vm.quiet_prompt();
        }
        vm.set_trace(opts.trace);

        let mut pending = Vec::new();
        for f in &opts.files {
            pending.extend_from_slice(format!("include {f}\n").as_bytes());
        }
        for e in &opts.evals {
            pending.extend_from_slice(e.as_bytes());
            pending.push(b'\n');
        }
        vm.feed(&pending);
        Ok(())
    }

    /// Run until `vm` needs input or halts; on `restart`, start a new
    /// machine the same way and hand it the input not yet read.
    fn until_blocked(&self, vm: &mut Machine, host: &mut StdHost) -> Result<Status, ExitCode> {
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
