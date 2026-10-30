use reva_memfs::memfs::MemFs;
use reva_vfs::dirs::FsDirs;
use reva_vfs::io::FsFiles;
use reva_vfs::meta::FsMeta;
use reva_vfs::types::{Access, SEEK_END, SEEK_SET, errno::*};

#[test]
fn memfs_round_trip() {
    let mut fs = MemFs::new();
    let fd = fs.open("/t.txt", Access::Create).unwrap();
    assert_eq!(fs.write(fd, b"hello").unwrap(), 5);
    assert_eq!(fs.read(fd, &mut [0; 4]), Err(EBADF)); // creat is write-only
    fs.close(fd).unwrap();
    let fd = fs.open("t.txt", Access::ReadWrite).unwrap();
    assert_eq!(fs.seek(fd, 0, SEEK_END), Ok(5));
    fs.write(fd, b" world").unwrap();
    assert_eq!(fs.seek(fd, 6, SEEK_SET), Ok(6));
    let mut buf = [0; 16];
    assert_eq!(fs.read(fd, &mut buf), Ok(5));
    assert_eq!(&buf[..5], b"world");
    assert_eq!(fs.fstat(fd).unwrap().size, 11);
    fs.close(fd).unwrap();
    assert_eq!(fs.close(fd), Err(EBADF));
    assert_eq!(
        fs.take_changes(),
        vec![("/t.txt".into(), Some(b"hello world".to_vec()))]
    );
    assert!(!fs.has_changes());
}

#[test]
fn memfs_dirs_and_names() {
    let mut fs = MemFs::new();
    assert_eq!(fs.open("/d/x", Access::Create), Err(ENOENT));
    fs.mkdir("/d").unwrap();
    assert_eq!(fs.mkdir("/d"), Err(EEXIST));
    fs.chdir("/d").unwrap();
    let fd = fs.open("x", Access::Create).unwrap();
    fs.close(fd).unwrap();
    assert_eq!(fs.list_dir(".").unwrap(), vec!["x".to_string()]);
    assert_eq!(fs.rmdir("/d"), Err(ENOTEMPTY));
    fs.rename("x", "/y").unwrap();
    assert_eq!(fs.stat("/y").unwrap().size, 0);
    assert_eq!(fs.stat("/d/x"), Err(ENOENT));
    fs.rmdir("/d").unwrap();
    fs.remove("/y").unwrap();
    assert_eq!(fs.remove("/y"), Err(ENOENT));
    let changes = fs.take_changes();
    assert!(changes.iter().all(|(_, d)| d.is_none()));
}

#[test]
fn memfs_sees_the_embedded_library() {
    if reva_embedded::embedded::lib_file("/reva/lib/util/locals").is_none() {
        return; // built without embed-lib
    }
    let mut fs = MemFs::new();
    assert!(fs.stat("/reva/lib").unwrap().dir);
    assert!(
        fs.list_dir("/reva/lib/util")
            .unwrap()
            .contains(&"locals".to_string())
    );
    assert!(fs.list_dir("/").unwrap().contains(&"reva".to_string()));
    let src = fs.read_whole("/reva/lib/util/locals").unwrap();
    assert!(!src.is_empty());
    assert_eq!(fs.remove("/reva/lib/util/locals"), Err(EACCES));
    // writing copies it; the copy shadows the embedded one
    let fd = fs.open("/reva/lib/util/locals", Access::ReadWrite).unwrap();
    fs.write(fd, b"|").unwrap();
    fs.close(fd).unwrap();
    assert_eq!(fs.get("/reva/lib/util/locals").unwrap()[0], b'|');
}
