use reva_desktop_pkg::brand::{branding, decode_png};
use reva_desktop_pkg::manifest::{Manifest, package_file, valid_id};
use reva_desktop_pkg::paths::{from_latin1, real_path, to_latin1, web_url};
use std::fs;
use std::path::Path;

#[test]
fn paths_stay_under_the_root() {
    let root = Path::new("/home/u/Reva");
    assert_eq!(real_path(root, "/a/b.f"), Some(root.join("a/b.f")));
    assert_eq!(real_path(root, "/../etc/passwd"), None);
    assert_eq!(real_path(root, "/a/../../x"), None);
    assert_eq!(real_path(root, "/reva/lib/x"), None);
    assert_eq!(real_path(root, "relative"), None);
}

#[test]
fn package_files_stay_in_the_package() {
    let dir = Path::new("/p");
    assert_eq!(
        package_file(dir, "icons/a.png"),
        Some(dir.join("icons/a.png"))
    );
    assert_eq!(package_file(dir, "../a.png"), None);
    assert_eq!(package_file(dir, "/etc/a.png"), None);
    assert_eq!(package_file(dir, ""), None);
}

#[test]
fn branding_defaults_and_checks() {
    let mut notes = Vec::new();
    let m: Manifest = serde_json::from_str(r#"{"name": "CoSy"}"#).unwrap();
    let b = branding(None, &m, &mut notes);
    assert_eq!(b.title, "CoSy on Reva");
    let m: Manifest =
        serde_json::from_str(r#"{"name": "CoSy", "title": "CoSy", "id": "co sy"}"#).unwrap();
    let b = branding(None, &m, &mut notes);
    assert_eq!((b.title.as_str(), b.id), ("CoSy", None));
    assert_eq!(notes.len(), 1);
    assert!(valid_id("cosy") && valid_id("org.example.Co_Sy-2"));
    assert!(!valid_id("") && !valid_id("a/b") && !valid_id(&"x".repeat(65)));
}

#[test]
fn icons_must_be_png_files_in_the_package() {
    let dir = std::env::temp_dir().join(format!("reva-desktop-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    // a 1x1 gray PNG with alpha, the smallest a real file can be
    let mut ok = Vec::new();
    let mut e = png::Encoder::new(&mut ok, 1, 1);
    e.set_color(png::ColorType::GrayscaleAlpha);
    e.write_header().unwrap().write_image_data(&[7, 9]).unwrap();
    assert_eq!(decode_png(&ok), Ok((vec![7, 7, 7, 9], 1, 1)));
    fs::write(dir.join("ok.png"), ok).unwrap();
    fs::write(dir.join("bad.png"), b"GIF89a").unwrap();
    let icon = |rel: &str, notes: &mut Vec<String>| {
        let m = Manifest {
            icon: Some(rel.into()),
            ..Default::default()
        };
        branding(Some(&dir), &m, notes).icon.is_some()
    };
    let mut notes = Vec::new();
    assert!(icon("ok.png", &mut notes) && notes.is_empty());
    assert!(!icon("bad.png", &mut notes) && !icon("gone.png", &mut notes));
    assert!(!icon("../ok.png", &mut notes));
    assert_eq!(notes.len(), 3);
    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn only_web_links_open() {
    assert!(web_url("https://cosy.com/CoSy/"));
    assert!(web_url("HTTP://example.org/x"));
    assert!(!web_url("file:///etc/passwd"));
    assert!(!web_url("javascript:alert(1)"));
    assert!(!web_url("https://a b"));
    assert!(!web_url("-a https://x"));
}

#[test]
fn latin1_round_trips_every_byte() {
    let all: Vec<u8> = (0..=255).collect();
    assert_eq!(from_latin1(&to_latin1(&all)), Some(all));
    assert_eq!(from_latin1("\u{100}"), None);
}
