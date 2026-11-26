use reva_hostpath::paths::{host_path, user_path, vm_path};
use reva_vm::vfs;
use std::path::PathBuf;

#[test]
fn windows_paths_round_trip_through_the_vm() {
    let v = vm_path(r"C:\Users\mike\cosy", true);
    assert_eq!(v, "/C:/Users/mike/cosy");
    assert_eq!(vm_path(r"\\?\D:\work", true), "/D:/work");
    let file = vfs::normalize(&v, "../reva/t.f");
    assert_eq!(file, "/C:/Users/mike/reva/t.f");
    assert_eq!(
        host_path(&file, true),
        PathBuf::from("C:/Users/mike/reva/t.f")
    );
    assert_eq!(host_path("/tmp/x", true), PathBuf::from("/tmp/x"));
}

#[test]
fn unix_paths_are_unchanged() {
    assert_eq!(vm_path("/home/mike", false), "/home/mike");
    assert_eq!(host_path("/C:/x", false), PathBuf::from("/C:/x"));
}

#[test]
fn a_windows_program_names_files_as_windows_does() {
    // what Git Bash passes on (D:/a/x.f) and what Windows programs write
    assert_eq!(user_path("D:/a/x.f", true), "/D:/a/x.f");
    assert_eq!(
        user_path(r"C:\Users\cosy\CoSy.f", true),
        "/C:/Users/cosy/CoSy.f"
    );
    assert_eq!(user_path(r".\sub\a.f", true), "./sub/a.f");
    assert_eq!(user_path("/C:/x", true), "/C:/x");
    assert_eq!(
        vfs::normalize("/D:/tmp", &user_path("D:/a/x.f", true)),
        "/D:/a/x.f"
    );
    // elsewhere a backslash is part of a name, and C: is a folder
    assert_eq!(user_path(r"a\b", false), r"a\b");
    assert_eq!(user_path("C:/x", false), "C:/x");
}
