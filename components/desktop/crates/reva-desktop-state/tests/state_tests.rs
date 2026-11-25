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

#[test]
fn about_reads_the_page_and_the_vm() {
    use reva_desktop_state::about::about;
    let page =
        b"<head><meta name=\"reva-build\" content=\"2026.1106.2 abc1234-dirty 2026-11-06\"></head>";
    let wasm = b"\0\0reva-fork 2026.1106.2 portable\0abc1234\0";
    assert_eq!(
        about("2026.1106.2", page, wasm),
        "app: reva-desktop 2026.1106.2\npage: 2026.1106.2 abc1234-dirty 2026-11-06\n\
         vm: reva-fork 2026.1106.2 portable\nvm build: abc1234 (in reva.wasm)\n"
    );
    assert!(about("1", b"", b"").contains("page: (none)"));
}

#[test]
fn the_page_gets_the_footer_settings() {
    // CoSy's asks 23 and 25: about.version leads the footer, sourceLinks
    // drops reva-fork's source links (www/about.js)
    let dir = package(
        "footer",
        r#"{"name": "cosy", "about": {"name": "CoSy", "version": "2026.1106.2"}, "sourceLinks": false}"#,
    );
    let o = Options {
        package: Some(dir.clone()),
        ..Default::default()
    };
    let p = build_state(o).unwrap().session.package.unwrap();
    assert_eq!(p["about"]["version"], "2026.1106.2");
    assert_eq!(p["sourceLinks"], false);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_user_folder_is_written_over_the_package() {
    use reva_desktop_pkg::snapshot::layered;
    let dir = package("layer", r#"{"name": "L", "seed": ["data.txt"]}"#);
    fs::write(dir.join("data.txt"), "start data").unwrap();
    fs::write(dir.join("code.f"), "package code").unwrap();
    let user = dir.with_extension("user");
    let _ = fs::remove_dir_all(&user);
    let o = Options {
        package: Some(dir.clone()),
        user: Some(user.clone()),
        ..Default::default()
    };
    let st = build_state(o).unwrap();
    assert_eq!(st.root, fs::canonicalize(&user).unwrap());
    assert_eq!(st.base, Some(fs::canonicalize(&dir).unwrap()));
    assert_eq!(
        fs::read_to_string(user.join("data.txt")).unwrap(),
        "start data"
    );
    fs::write(user.join("code.f"), "user code").unwrap();
    let files = layered(st.base.as_deref(), &st.root, &mut Vec::new());
    let code: Vec<_> = files.iter().filter(|(vm, _)| vm == "/code.f").collect();
    assert_eq!(code.len(), 1);
    assert_eq!(code[0].1, "user code");
    assert!(files.iter().any(|(vm, _)| vm == "/reva-package.json"));
    fs::remove_dir_all(dir).unwrap();
    fs::remove_dir_all(user).unwrap();
}

#[test]
fn a_package_without_a_user_folder_is_its_own_root() {
    let dir = package("own", r#"{"name": "Own"}"#);
    let o = Options {
        package: Some(dir.clone()),
        ..Default::default()
    };
    let st = build_state(o).unwrap();
    assert_eq!((st.root, st.base), (fs::canonicalize(&dir).unwrap(), None));
    fs::remove_dir_all(dir).unwrap();
}
