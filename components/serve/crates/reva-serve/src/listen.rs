//! Listening on loopback: binding (or finding a copy already running),
//! probing, and serving connections.

use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::site::Site;
use reva_serve_http::{http, site};

/// How many ports after the preferred one [`bind`] tries.
pub const FALLBACKS: u16 = 20;
/// At most this many connections are served at once; more wait in the
/// listen queue.  A page opens about six.
const MAX_CONNECTIONS: usize = 32;

/// What [`bind`] found.
pub enum Bound {
    /// Listening on this port (the preferred one, or a fallback).
    Listening(TcpListener, u16),
    /// A copy of this app already serves on this port.
    Running(u16),
}

/// Listen on `127.0.0.1:port`, or find a copy of `site` already there, or
/// fall back to the next free port (up to [`FALLBACKS`] more).  `port` 0
/// takes any free port (for tests).
pub fn bind(site: &Site, port: u16) -> io::Result<Bound> {
    if port == 0 {
        let l = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        let p = l.local_addr()?.port();
        return Ok(Bound::Listening(l, p));
    }
    let mut last = None;
    for p in port..=port.saturating_add(FALLBACKS) {
        match TcpListener::bind((Ipv4Addr::LOCALHOST, p)) {
            Ok(l) => return Ok(Bound::Listening(l, p)),
            Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                // any version of this app counts: its files are at this port
                if p == port && probe(p).is_some_and(|s| same_app(&s, site)) {
                    return Ok(Bound::Running(p));
                }
                last = Some(e);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or_else(|| io::Error::other("no port")))
}

/// Whether a [`site::PROBE`] answer comes from this app (any version).
fn same_app(answer: &str, site: &Site) -> bool {
    answer.starts_with(&format!("reva-serve {} ", site.name))
}

/// Ask whatever listens on `port` for [`site::PROBE`]; its body if it answers 200.
pub fn probe(port: u16) -> Option<String> {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_millis(500)).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(1))).ok()?;
    let req = format!(
        "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n",
        site::PROBE
    );
    s.write_all(req.as_bytes()).ok()?;
    let mut buf = Vec::new();
    s.take(4096).read_to_end(&mut buf).ok()?;
    let text = String::from_utf8(buf).ok()?;
    let (head, body) = text.split_once("\r\n\r\n")?;
    head.starts_with("HTTP/1.1 200").then(|| body.to_string())
}

/// Serve `site` on `listener` until the process ends, one thread per
/// connection (at most [`MAX_CONNECTIONS`] at once).
pub fn serve(listener: TcpListener, site: &'static Site, port: u16) -> ! {
    let busy = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        while busy.load(Ordering::Acquire) >= MAX_CONNECTIONS {
            std::thread::sleep(Duration::from_millis(5));
        }
        busy.fetch_add(1, Ordering::AcqRel);
        let busy = Arc::clone(&busy);
        std::thread::spawn(move || {
            // A client that goes away mid-response is not our problem.
            let _ = http::handle(stream, site, port);
            busy.fetch_sub(1, Ordering::AcqRel);
        });
    }
    unreachable!("TcpListener::incoming never ends")
}
