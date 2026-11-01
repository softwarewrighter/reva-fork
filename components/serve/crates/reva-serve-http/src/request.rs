//! Parsing a request, and checking its `Host`.

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
