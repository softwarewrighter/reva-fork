//! `reva-serve` (and builds of it such as `cosy-serve`): serve the embedded
//! site on 127.0.0.1 and open the browser on it.  See the library docs and
//! docs/serve.md.

use std::process::ExitCode;

use reva_serve::site::{embedded, SITE};
use reva_serve::{bind, open, serve, Bound, FALLBACKS};

fn usage() -> String {
    format!(
        "usage: {name}-serve [options]

Serves {name} on http://127.0.0.1:{port}/ and opens it in the browser.
Your files are kept by the browser for that address; Ctrl-C stops the server.

  --port N     use port N instead of {port} (files saved at another port are kept apart)
  --no-open    do not open the browser
  --list       list the embedded files and exit
  -h, --help   this text
  -V, --version
",
        name = embedded::NAME,
        port = embedded::PORT
    )
}

struct Options {
    port: u16,
    open: bool,
}

fn parse_args() -> Result<Options, String> {
    let mut o = Options {
        port: embedded::PORT,
        open: true,
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--port" => {
                let v = args.next().ok_or("--port needs a value")?;
                o.port = v
                    .parse()
                    .map_err(|_| format!("--port {v}: not a port number"))?;
            }
            "--no-open" => o.open = false,
            "--list" => {
                for (path, bytes) in SITE.files {
                    println!("{:>9}  {path}", bytes.len());
                }
                std::process::exit(0);
            }
            "-h" | "--help" => {
                print!("{}", usage());
                std::process::exit(0);
            }
            "-V" | "--version" => {
                print!("{}", SITE.probe());
                std::process::exit(0);
            }
            // macOS passes -psn_... when launched from Finder
            s if s.starts_with("-psn_") => {}
            s => return Err(format!("unknown option {s}")),
        }
    }
    Ok(o)
}

fn launch(url: &str, open_it: bool) {
    if open_it {
        if let Err(e) = open::browser(url) {
            eprintln!("{}-serve: {e}", SITE.name);
        }
    }
}

fn main() -> ExitCode {
    let o = match parse_args() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{}-serve: {e}\n{}", SITE.name, usage());
            return ExitCode::from(2);
        }
    };
    let name = SITE.name;
    match bind(&SITE, o.port) {
        Ok(Bound::Running(port)) => {
            let url = format!("http://127.0.0.1:{port}/");
            println!("{name} is already running at {url}");
            launch(&url, o.open);
            ExitCode::SUCCESS
        }
        Ok(Bound::Listening(listener, port)) => {
            let url = format!("http://127.0.0.1:{port}/");
            if port != o.port {
                eprintln!(
                    "{name}-serve: port {} is taken by another program; using {port}.\n\
                     Files you saved at port {} are not visible here (the browser keeps them per port).",
                    o.port, o.port
                );
            }
            println!("{name} at {url}  (Ctrl-C to stop)");
            launch(&url, o.open);
            // Ctrl-C ends the process the default way: there is nothing to
            // flush, since the browser holds the user's files.
            serve(listener, &SITE, port)
        }
        Err(e) => {
            eprintln!(
                "{name}-serve: no free port in {}..={}: {e}",
                o.port,
                o.port.saturating_add(FALLBACKS)
            );
            ExitCode::FAILURE
        }
    }
}
