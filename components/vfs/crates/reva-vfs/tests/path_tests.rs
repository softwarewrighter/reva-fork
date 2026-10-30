use reva_vfs::types::normalize;

#[test]
fn paths_normalize() {
    assert_eq!(normalize("/", "a/b"), "/a/b");
    assert_eq!(normalize("/x/y", "../z/./w"), "/x/z/w");
    assert_eq!(normalize("/x", "/a//b/"), "/a/b");
    assert_eq!(normalize("/", ".."), "/");
}
