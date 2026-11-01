use reva_serve_http::site::{Site, mime};

static FILES: &[(&str, &[u8])] = &[
    ("a.js", b"js"),
    ("index.html", b"top"),
    ("pkg/index.html", b"pkg"),
    ("pkg/x.f", b"x"),
];
const SITE: Site = Site {
    name: "t",
    files: FILES,
};

#[test]
fn finds_files_and_index_pages() {
    assert_eq!(SITE.lookup("/").map(|f| f.1), Some(&b"top"[..]));
    assert_eq!(SITE.lookup("/pkg/").map(|f| f.1), Some(&b"pkg"[..]));
    assert_eq!(SITE.lookup("/pkg/x.f").map(|f| f.0), Some("pkg/x.f"));
    assert_eq!(SITE.lookup("/pkg"), None);
    assert_eq!(SITE.lookup("/nope"), None);
    assert_eq!(SITE.lookup("/pkg/../a.js"), None);
    assert_eq!(SITE.lookup("a.js"), None);
}

#[test]
fn media_types() {
    assert_eq!(mime("reva.wasm"), "application/wasm");
    assert_eq!(mime("x/CoSy.f"), "text/plain; charset=utf-8");
    assert_eq!(mime("INDEX.HTML"), "text/html; charset=utf-8");
    assert_eq!(mime("noext"), "application/octet-stream");
}
