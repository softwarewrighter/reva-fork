//! The server end to end, on real loopback sockets.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};

use reva_serve::site::SITE;
use reva_serve::{bind, probe, serve, Bound};

/// Serve the embedded site on a free port, in the background.
fn start() -> u16 {
    let Ok(Bound::Listening(l, port)) = bind(&SITE, 0) else {
        panic!("no port")
    };
    std::thread::spawn(move || serve(l, &SITE, port));
    port
}

fn get(port: u16, path: &str, host: &str) -> (String, Vec<u8>) {
    let mut s = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).unwrap();
    write!(s, "GET {path} HTTP/1.1\r\nHost: {host}\r\n\r\n").unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).unwrap();
    let at = buf.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    (
        String::from_utf8_lossy(&buf[..at]).into_owned(),
        buf[at + 4..].to_vec(),
    )
}

#[test]
fn serves_the_embedded_site() {
    let port = start();
    let host = format!("127.0.0.1:{port}");
    let (head, body) = get(port, "/?canvas=split", &host);
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    assert!(head.contains("Content-Type: text/html"), "{head}");
    assert!(String::from_utf8_lossy(&body).contains("<!doctype html>"));
    let (head, _) = get(port, "/reva.js", &format!("localhost:{port}"));
    assert!(head.contains("text/javascript"), "{head}");
    let (head, _) = get(port, "/no-such-file", &host);
    assert!(head.starts_with("HTTP/1.1 404"), "{head}");
    let (head, _) = get(port, "/", "rebind.example:80");
    assert!(head.starts_with("HTTP/1.1 421"), "{head}");
    assert_eq!(probe(port), Some(SITE.probe()));
}

#[test]
fn a_second_launch_finds_the_first() {
    // Find a port that is free now, serve on it the way the binary does.
    let port = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let Ok(Bound::Listening(l, p)) = bind(&SITE, port) else {
        panic!("not free")
    };
    assert_eq!(p, port);
    std::thread::spawn(move || serve(l, &SITE, port));
    assert!(matches!(bind(&SITE, port), Ok(Bound::Running(p)) if p == port));
}

#[test]
fn another_program_on_the_port_means_the_next_one() {
    let other = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let port = other.local_addr().unwrap().port();
    // `other` never answers; the probe times out and bind moves on.
    std::thread::spawn(move || {
        let _keep = other.accept();
        std::thread::sleep(std::time::Duration::from_secs(5));
    });
    match bind(&SITE, port) {
        Ok(Bound::Listening(_, p)) => assert!(p > port),
        Ok(Bound::Running(_)) => panic!("mistook another program for this one"),
        Err(e) => panic!("{e}"),
    }
}
