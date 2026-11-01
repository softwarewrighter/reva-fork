use reva_serve_http::request::{Refusal, host_ok, parse};

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
