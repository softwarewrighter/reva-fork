use reva_desktop_pkg::manifest::Manifest;
use reva_desktop_user::user::{installed_package, seed, user_dir};
use std::fs;
use std::path::{Path, PathBuf};

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("reva-user-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn manifest(json: &str) -> Manifest {
    serde_json::from_str(json).unwrap()
}

#[test]
fn the_user_folder_is_named_by_the_manifest_when_installed() {
    let docs = Path::new("/home/u/Documents");
    let mut notes = Vec::new();
    let m = manifest(r#"{"name": "Demo", "userFolder": "Demo Files"}"#);
    assert_eq!(user_dir(None, false, &m, docs, &mut notes), None);
    assert_eq!(
        user_dir(None, true, &m, docs, &mut notes),
        Some(docs.join("Demo Files"))
    );
    let own = Path::new("/tmp/mine");
    assert_eq!(
        user_dir(Some(own), false, &m, docs, &mut notes),
        Some(own.to_path_buf())
    );
    assert!(notes.is_empty());
}

#[test]
fn a_bad_user_folder_falls_back_to_the_name() {
    let docs = Path::new("/d");
    let mut notes = Vec::new();
    let m = manifest(r#"{"name": "Demo", "userFolder": "../escape"}"#);
    assert_eq!(
        user_dir(None, true, &m, docs, &mut notes),
        Some(docs.join("Demo"))
    );
    assert_eq!(notes.len(), 1);
    let m = manifest(r#"{"name": "a/b"}"#);
    assert_eq!(
        user_dir(None, true, &m, docs, &mut notes),
        Some(docs.join("Reva"))
    );
}

#[test]
fn seeds_are_copied_once_and_never_overwritten() {
    let pkg = temp("seed-pkg");
    let user = temp("seed-user");
    fs::create_dir_all(pkg.join("data")).unwrap();
    fs::write(pkg.join("data/start.txt"), "package's").unwrap();
    let seeds = vec!["data/start.txt".to_string(), "../x".to_string()];
    let mut notes = Vec::new();
    seed(&pkg, &user, &seeds, &mut notes);
    assert_eq!(
        fs::read_to_string(user.join("data/start.txt")).unwrap(),
        "package's"
    );
    assert_eq!(notes.len(), 1);
    fs::write(user.join("data/start.txt"), "user's").unwrap();
    seed(&pkg, &user, &seeds[..1], &mut notes);
    assert_eq!(
        fs::read_to_string(user.join("data/start.txt")).unwrap(),
        "user's"
    );
    fs::remove_dir_all(pkg).unwrap();
    fs::remove_dir_all(user).unwrap();
}

#[test]
fn an_installed_package_needs_its_manifest() {
    let res = temp("res");
    assert_eq!(installed_package(Some(res.clone())), None);
    fs::create_dir_all(res.join("package")).unwrap();
    fs::write(res.join("package/reva-package.json"), "{}").unwrap();
    assert_eq!(
        installed_package(Some(res.clone())),
        Some(res.join("package"))
    );
    assert_eq!(installed_package(None), None);
    fs::remove_dir_all(res).unwrap();
}
