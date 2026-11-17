use reva_embedded::embedded::{RENAME_LINUX, RENAME_WORD, REVA_F, boot_source};

#[test]
fn rename_takes_two_strings_at_boot() {
    assert!(REVA_F.contains(RENAME_LINUX), "reva.f's Linux rename moved");
    assert!(!boot_source().contains(RENAME_LINUX));
    assert_eq!(boot_source().matches(RENAME_WORD).count(), 1);
}
