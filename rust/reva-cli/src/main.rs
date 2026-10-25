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

use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reva_vm::vfs::FileSystem;
use reva_vm::{embedded, Host, Machine, Status};

const USAGE: &str = "\
usage: reva [options] [file ...]

  -e, --eval TEXT      evaluate TEXT after the files (repeatable)
      --boot PATH      boot from PATH instead of the embedded reva.f
      --no-boot        start from the bare core (no reva.f at all)
      --image PATH     start from a saved image instead of booting
      --save-image PATH  after files and -e, save an image and exit
      --show-boot      print what booting printed (progress digits, errors)
      --trace          trace every VM step to stderr (very slow)
      --allow-shell    let `shell` (C `system`) run OS commands; off by default
  -h, --help           this text
  -V, --version        version
";

mod fs;

struct StdHost {
    out: io::BufWriter<io::Stdout>,
    fs: fs::StdFs,
    allow_shell: bool,
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
        let _ = self.out.write_all(bytes);
    }

    os_services!();

    /// VM notes go to stderr, after what Forth printed so far.
    fn diag(&mut self, line: &str) {
        let _ = self.out.flush();
        eprint!("{line}");
    }

    /// `system(3)`: `sh -c cmd`, returning the wait status as C does.
    /// Only with `--allow-shell`: Reva programs run OS commands freely, so
    /// that is a choice for whoever starts them, not for the program.
    fn system(&mut self, cmd: &str) -> Option<i32> {
        if !self.allow_shell {
            return None;
        }
        let _ = self.out.flush();
        let status = std::process::Command::new("sh").arg("-c").arg(cmd).status();
        Some(match status {
            Ok(s) => wait_status(s),
            Err(_) => 127 << 8,
        })
    }

    fn read_input_blocking(&mut self) -> Option<Vec<u8>> {
        let _ = self.out.flush();
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(_) => Some(line.into_bytes()),
            Err(_) => None,
        }
    }
}

/// Collects output during boot so the progress digits stay quiet.
struct Capture {
    out: Vec<u8>,
    fs: fs::StdFs,
}

impl Host for Capture {
    fn write(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    os_services!();

    fn diag(&mut self, line: &str) {
        eprint!("{line}");
    }
}

#[cfg(unix)]
fn wait_status(s: std::process::ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    s.into_raw()
}

#[cfg(not(unix))]
fn wait_status(s: std::process::ExitStatus) -> i32 {
    s.code().unwrap_or(-1)
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
    allow_shell: bool,
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
            "--allow-shell" => o.allow_shell = true,
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
    let mut vm = match &opts.image {
        Some(path) => {
            let bytes = match std::fs::read(path) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("reva: {path}: {e}");
                    return ExitCode::from(2);
                }
            };
            match Machine::load_image(&bytes, &["reva"]) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("reva: {path}: {e}");
                    return ExitCode::from(2);
                }
            }
        }
        None if opts.no_boot => Machine::new(&["reva"]),
        None => Machine::new(&["reva", &boot_path]),
    };

    // Boot quietly.  reva.f prints progress marks, then (as the restarted
    // system) its banner and first prompt; anything else means the boot
    // went wrong, so show it.
    let mut capture = Capture {
        out: Vec::new(),
        fs: fs::StdFs::default(),
    };
    if let Status::Halted(code) = vm.run_until_blocked(&mut capture) {
        io::stdout().write_all(&capture.out).ok();
        eprintln!("reva: halted during boot");
        return exit_code(code);
    }
    let boot_out = String::from_utf8_lossy(&capture.out).into_owned();
    let (marks, rest) = embedded::split_boot_output(&boot_out);
    let normal = !rest.contains("***") && (rest.is_empty() || rest.starts_with("Reva "));
    if opts.show_boot || !normal {
        eprint!("{marks}{rest}");
        if !boot_out.ends_with('\n') && !boot_out.is_empty() {
            eprintln!();
        }
    }
    let interactive = io::stdin().is_terminal() && opts.save_image.is_none();
    if interactive {
        // The banner and first `ok>` prompt, as Reva showed them.
        if normal && !opts.show_boot {
            print!("{rest}");
        }
    } else {
        vm.quiet_prompt();
    }
    vm.set_trace(opts.trace);

    let mut host = StdHost {
        out: io::BufWriter::new(io::stdout()),
        fs: capture.fs,
        allow_shell: opts.allow_shell,
    };
    let mut pending = Vec::new();
    for f in &opts.files {
        pending.extend_from_slice(format!("include {f}\n").as_bytes());
    }
    for e in &opts.evals {
        pending.extend_from_slice(e.as_bytes());
        pending.push(b'\n');
    }
    vm.feed(&pending);
    let status = vm.run_until_blocked(&mut host);
    let _ = host.out.flush();
    if let Status::Halted(code) = status {
        return exit_code(code);
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
        let status = vm.run_until_blocked(&mut host);
        let _ = host.out.flush();
        return match status {
            Status::Halted(code) => exit_code(code),
            _ => ExitCode::SUCCESS,
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
        let status = vm.run_until_blocked(&mut host);
        let _ = host.out.flush();
        if let Status::Halted(code) = status {
            return exit_code(code);
        }
    }
}
