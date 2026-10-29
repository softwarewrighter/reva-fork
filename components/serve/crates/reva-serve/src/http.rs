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

use crate::site::{self, Site};

/// The longest request head read; anything longer is refused.
const MAX_HEAD: usize = 16 << 10;
/// A client that sends nothing for this long is dropped.
const TIMEOUT: Duration = Duration::from_secs(10);

/// A parsed request line and the headers we use.
#[derive(Debug, PartialEq, Eq)]
pub struct Request {
    /// `GET` or `HEAD` (others are refused while parsing).
    pub method: String,
    /// The path, percent-decoded, without the query.
    pub path: String,
    /// The `Host` header, if any.
    pub host: Option<String>,
}

/// Why a request was not served from the site.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    /// 400: not a request we understand.
    Bad,
    /// 405: a method other than GET or HEAD.
    Method,
    /// 421: a `Host` that is not this server's loopback address.
    Host,
}

/// Parse a request head (the request line and headers, CRLF or LF).
pub fn parse(head: &str) -> Result<Request, Refusal> {
    let mut lines = head.lines();
    let mut first = lines.next().ok_or(Refusal::Bad)?.split(' ');
    let (method, target) = (
        first.next().unwrap_or(""),
        first.next().ok_or(Refusal::Bad)?,
    );
    if !first.next().is_some_and(|v| v.starts_with("HTTP/1.")) {
        return Err(Refusal::Bad);
    }
    if method != "GET" && method != "HEAD" {
        return Err(Refusal::Method);
    }
    let raw = target.split(['?', '#']).next().unwrap_or("");
    let path = percent_decode(raw).ok_or(Refusal::Bad)?;
    if !path.starts_with('/') {
        return Err(Refusal::Bad);
    }
    let host = lines
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.trim().eq_ignore_ascii_case("host"))
        .map(|(_, v)| v.trim().to_ascii_lowercase());
    Ok(Request {
        method: method.into(),
        path,
        host,
    })
}

/// Decode `%XX` escapes; `None` for a bad escape or a result that is not UTF-8.
pub fn percent_decode(s: &str) -> Option<String> {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            let hex = s.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Whether `host` (a `Host` header) names this server: `127.0.0.1:PORT`
/// or `localhost:PORT`.
pub fn host_ok(host: Option<&str>, port: u16) -> bool {
    let Some(h) = host else { return false };
    let Some((name, p)) = h.rsplit_once(':') else {
        return false;
    };
    p.parse() == Ok(port) && matches!(name, "127.0.0.1" | "localhost")
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_get_and_strips_the_query() {
        let r = parse("GET /a%20b.f?x=1 HTTP/1.1\r\nHost: 127.0.0.1:8421\r\n\r\n").unwrap();
        assert_eq!(r.method, "GET");
        assert_eq!(r.path, "/a b.f");
        assert_eq!(r.host.as_deref(), Some("127.0.0.1:8421"));
    }

    #[test]
    fn refuses_what_it_does_not_serve() {
        assert_eq!(parse("POST / HTTP/1.1\r\n\r\n"), Err(Refusal::Method));
        assert_eq!(parse("GET /\r\n\r\n"), Err(Refusal::Bad));
        assert_eq!(parse("GET x HTTP/1.1\r\n\r\n"), Err(Refusal::Bad));
        assert_eq!(parse("GET /%zz HTTP/1.1\r\n\r\n"), Err(Refusal::Bad));
        assert_eq!(parse(""), Err(Refusal::Bad));
    }

    #[test]
    fn only_loopback_hosts_on_our_port() {
        assert!(host_ok(Some("127.0.0.1:8421"), 8421));
        assert!(host_ok(Some("localhost:8421"), 8421));
        assert!(!host_ok(Some("localhost:8422"), 8421));
        assert!(!host_ok(Some("evil.example:8421"), 8421));
        assert!(!host_ok(Some("127.0.0.1"), 8421));
        assert!(!host_ok(None, 8421));
    }
}
