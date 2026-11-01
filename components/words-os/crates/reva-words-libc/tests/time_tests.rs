use reva_words_libc::time::tm_from_unix;

#[test]
fn tm_known_dates() {
    assert_eq!(tm_from_unix(0), [0, 0, 0, 1, 0, 70, 4, 0, 0]);
    // 2000-02-29 12:34:56 UTC, a Tuesday, day 59 of a leap year
    assert_eq!(
        tm_from_unix(951_827_696),
        [56, 34, 12, 29, 1, 100, 2, 59, 0]
    );
    // 2026-10-25 00:00:00 UTC, a Sunday
    assert_eq!(
        tm_from_unix(1_792_886_400),
        [0, 0, 0, 25, 9, 126, 0, 297, 0]
    );
}
