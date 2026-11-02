use reva_desktop_state::options::Options;
use reva_desktop_state::state::build_state;
use std::fs;
use std::path::PathBuf;

/// A fresh package folder under the system temp directory.
fn package(name: &str, manifest: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("reva-state-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("reva-package.json"), manifest).unwrap();
    dir
}

#[test]
fn a_package_image_is_where_it_starts() {
    let dir = package("img", r#"{"name": "Img", "image": "app.img"}"#);
    fs::write(dir.join("app.img"), b"image").unwrap();
    let o = Options {
        package: Some(dir.clone()),
        ..Default::default()
    };
    let st = build_state(o).unwrap();
    assert!(st.session.image);
    assert_eq!(
        st.image,
        Some(fs::canonicalize(&dir).unwrap().join("app.img"))
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn image_option_wins_and_a_missing_image_is_a_note() {
    let dir = package("noimg", r#"{"name": "Img", "image": "gone.img"}"#);
    let o = Options {
        package: Some(dir.clone()),
        ..Default::default()
    };
    let st = build_state(o).unwrap();
    assert!(!st.session.image && st.image.is_none());
    assert!(st.session.notes.iter().any(|n| n.contains("not found")));
    let own = dir.join("own.img");
    fs::write(&own, b"image").unwrap();
    let o = Options {
        package: Some(dir.clone()),
        image: Some(own.clone()),
        ..Default::default()
    };
    assert_eq!(build_state(o).unwrap().image, Some(own));
    fs::remove_dir_all(dir).unwrap();
}
