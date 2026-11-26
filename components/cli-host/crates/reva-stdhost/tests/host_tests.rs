use reva_stdhost::host::wait_status;

/// `exit 3` is 3 * 256 on every OS, as Unix's `system` gives it.
#[test]
fn an_exit_code_is_a_unix_wait_status() {
    #[cfg(unix)]
    let s = std::os::unix::process::ExitStatusExt::from_raw(3 << 8);
    #[cfg(windows)]
    let s = std::os::windows::process::ExitStatusExt::from_raw(3);
    assert_eq!(wait_status(s), 3 << 8);
}
