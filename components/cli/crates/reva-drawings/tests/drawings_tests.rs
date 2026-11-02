use reva_canvas::canvas::Canvas;
use reva_drawings::drawings::{Drawings, Format, Mode, parse_mode, resolve};
use reva_drawings::inline::base64;
use reva_drawings::save::save_new;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("reva-drawings-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn modes_resolve_by_terminal() {
    assert_eq!(resolve(Mode::Auto, true, true), (Some(Format::Png), true));
    assert_eq!(resolve(Mode::Auto, true, false), (Some(Format::Svg), false));
    assert_eq!(
        resolve(Mode::Auto, false, true),
        (None, false),
        "redirected: no files"
    );
    assert_eq!(resolve(Mode::Svg, false, false), (Some(Format::Svg), false));
    assert_eq!(resolve(Mode::Png, false, true), (Some(Format::Png), false));
    assert_eq!(resolve(Mode::Off, true, true), (None, false));
    assert!(parse_mode("gif").is_err());
}

#[test]
fn new_files_never_overwrite() {
    let d = temp_dir("save");
    std::fs::write(d.join("turtle-001.svg"), "mine").unwrap();
    assert_eq!(
        save_new(&d, "turtle", "svg", b"a").unwrap(),
        "turtle-002.svg"
    );
    assert_eq!(
        save_new(&d, "turtle", "svg", b"b").unwrap(),
        "turtle-003.svg"
    );
    assert_eq!(
        std::fs::read_to_string(d.join("turtle-001.svg")).unwrap(),
        "mine"
    );
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn only_a_changed_drawing_is_written() {
    let d = temp_dir("after");
    let mut dr = Drawings {
        format: Some(Format::Svg),
        ..Drawings::default()
    };
    let mut c = Canvas::new();
    assert!(dr.after(&c, &d).unwrap().is_none(), "nothing drawn");
    c.forward(100.0);
    assert_eq!(dr.after(&c, &d).unwrap().unwrap().name, "turtle-001.svg");
    assert!(dr.after(&c, &d).unwrap().is_none(), "unchanged");
    c.right(90.0);
    c.forward(50.0);
    let w = dr.after(&c, &d).unwrap().unwrap();
    assert_eq!((w.name.as_str(), w.inline.as_str()), ("turtle-002.svg", ""));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn base64_pads() {
    assert_eq!(base64(b"Ma"), "TWE=");
    assert_eq!(base64(b"Man"), "TWFu");
    assert_eq!(base64(b"M"), "TQ==");
}
