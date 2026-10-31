use reva_hostpath::paths::{host_path, vm_path};
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
