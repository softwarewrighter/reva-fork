//! Just enough HTTP/1.1 to serve static files to one local browser.
//!
//! One request per connection (`Connection: close`): the page loads about a
//! dozen files once, so keep-alive would save nothing worth its code.
//! Only `GET` and `HEAD`.  The `Host` header must name the loopback address
//! the server is on, which stops DNS-rebinding pages from reading the site
//! through a name of their own.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::request::{Refusal, Request, host_ok, parse};
use crate::site::{self, Site};

/// The longest request head read; anything longer is refused.
const MAX_HEAD: usize = 16 << 10;
/// A client that sends nothing for this long is dropped.
const TIMEOUT: Duration = Duration::from_secs(10);

/// Serve one connection: read a request, write the response, close.
pub fn handle(mut stream: TcpStream, site: &Site, port: u16) -> io::Result<()> {
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    let head = read_head(&mut stream)?;
    let req = parse(&head).and_then(|r| {
        if host_ok(r.host.as_deref(), port) {
            Ok(r)
        } else {
            Err(Refusal::Host)
        }
    });
    match req {
        Ok(r) => respond(&mut stream, &r, site),
        Err(Refusal::Bad) => status(&mut stream, "400 Bad Request"),
        Err(Refusal::Method) => status(&mut stream, "405 Method Not Allowed"),
        Err(Refusal::Host) => status(&mut stream, "421 Misdirected Request"),
    }
}

fn read_head(stream: &mut TcpStream) -> io::Result<String> {
    let mut r = BufReader::new(stream.take(MAX_HEAD as u64));
    let mut head = String::new();
    loop {
        let n = r.read_line(&mut head)?;
        if n == 0 || head.ends_with("\r\n\r\n") || head.ends_with("\n\n") {
            return Ok(head);
        }
        if head.len() >= MAX_HEAD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request head too long",
            ));
        }
    }
}

fn respond(stream: &mut TcpStream, r: &Request, site: &Site) -> io::Result<()> {
    if r.path == site::PROBE {
        return send(
            stream,
            r,
            "200 OK",
            "text/plain; charset=utf-8",
            site.probe().as_bytes(),
        );
    }
    match site.lookup(&r.path) {
        Some((path, body)) => send(stream, r, "200 OK", site::mime(path), body),
        None => status(stream, "404 Not Found"),
    }
}

fn status(stream: &mut TcpStream, line: &str) -> io::Result<()> {
    let r = Request {
        method: "GET".into(),
        path: String::new(),
        host: None,
    };
    send(
        stream,
        &r,
        line,
        "text/plain; charset=utf-8",
        format!("{line}\n").as_bytes(),
    )
}

fn send(
    stream: &mut TcpStream,
    r: &Request,
    line: &str,
    mime: &str,
    body: &[u8],
) -> io::Result<()> {
    // no-cache: the browser revalidates, so a rebuilt binary is seen at once
    let head = format!(
        "HTTP/1.1 {line}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\n\
         Cache-Control: no-cache\r\nX-Content-Type-Options: nosniff\r\n\
         Referrer-Policy: no-referrer\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    if r.method != "HEAD" {
        stream.write_all(body)?;
    }
    stream.flush()
}
